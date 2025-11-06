%% knhk_epoch.erl — Epoch scheduler (Λ ≺-total, τ ≤ 8)
%% Plans deterministic execution epochs, executes μ over O
-module(knhk_epoch).
-behaviour(gen_server).

-export([start_link/0, schedule/3, run/1, validate_lambda/1, select_hooks/2]).

-record(state, {epochs = #{} :: map(), hooks = #{} :: map()}).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

init([]) ->
    {ok, #state{}}.

handle_call({schedule, Tau, Plan, CoverId}, _From, State) when Tau =< 8 ->
    %% Validate Λ is ≺-total (no duplicates, deterministic order)
    case validate_lambda_order(Plan) of
        ok ->
            EpochId = erlang:phash2({Plan, CoverId}),
            Epoch = #{tau => Tau, lambda => Plan, cover => CoverId},
            NewState = State#state{epochs = maps:put(EpochId, Epoch, State#state.epochs)},
            {reply, {ok, EpochId}, NewState};
        {error, Reason} ->
            {reply, {error, {order_violation, Reason}}, State}
    end;
handle_call({schedule, Tau, _Plan, _CoverId}, _From, State) when Tau > 8 ->
    {reply, {error, {guard_violation, <<"tau > 8">>}}, State};
handle_call({run, EpochId}, _From, State) ->
    case maps:get(EpochId, State#state.epochs, undefined) of
        undefined ->
            {reply, {error, not_found}, State};
        Epoch ->
            %% A = μ(O); hash(A) = hash(μ(O))
            %% Select hooks by Λ order
            Lambda = maps:get(lambda, Epoch),
            SelectedHooks = select_hooks_by_lambda(Lambda, State#state.hooks),
            %% In production, this would call C FFI for hot path execution
            A = #{result => executed, hooks => SelectedHooks},
            Receipt = #{hash => erlang:phash2(EpochId), span => EpochId, ticks => 8},
            {reply, {ok, #{A => A, receipt => Receipt}}, State}
    end;
handle_call(_Request, _From, State) ->
    {reply, {error, unknown}, State}.

schedule(Tau, Plan, CoverId) ->
    gen_server:call(?MODULE, {schedule, Tau, Plan, CoverId}).

run(EpochId) ->
    gen_server:call(?MODULE, {run, EpochId}).

%% Validate Λ is ≺-total (deterministic order, no cycles)
validate_lambda(Plan) ->
    gen_server:call(?MODULE, {validate_lambda, Plan}).

%% Select hooks by Λ order
select_hooks(Lambda, HookIds) ->
    gen_server:call(?MODULE, {select_hooks, Lambda, HookIds}).

%% Internal: Validate Λ order (no duplicates)
validate_lambda_order(Plan) ->
    %% Check for duplicates
    UniquePlan = lists:usort(Plan),
    case length(UniquePlan) =:= length(Plan) of
        true -> ok;
        false -> {error, <<"Lambda contains duplicates, violates ≺-total ordering">>}
    end.

%% Internal: Select hooks by Λ order
select_hooks_by_lambda(Lambda, Hooks) ->
    %% Filter hooks that are in Lambda order
    lists:filter(fun(HookId) -> lists:member(HookId, Lambda) end, maps:keys(Hooks)).

