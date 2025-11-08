use knhk_workflow_engine::api::rest::RestApiServer;

let server = RestApiServer::new(Arc::clone(&engine));
let router = server.router();

// Use with axum server
let app = Router::new()
    .nest("/api/v1", router);