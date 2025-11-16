/**
 * KNHK WASM Node.js Example
 *
 * Demonstrates executing workflows using the KNHK WASM module in Node.js
 */

import { WasmWorkflowEngine, WasmEngineConfig } from '../../wasm-dist/nodejs/knhk_wasm.js';

async function main() {
    console.log('🚀 KNHK WASM Node.js Example\n');

    // Create engine with custom configuration
    const config = new WasmEngineConfig();
    config.set_max_workflows(50);
    config.set_enable_telemetry(true);
    config.set_timeout_ms(10000); // 10 seconds

    const engine = WasmWorkflowEngine.with_config(config);
    console.log('✅ Engine initialized\n');

    // Example 1: Simple sequence workflow
    console.log('Example 1: Sequence Workflow');
    console.log('==============================');

    const sequenceWorkflow = {
        id: 'user-onboarding',
        pattern: 'Sequence',
        tasks: [
            {
                id: 'validate-input',
                type: 'validate',
                config: {
                    required: ['email', 'name']
                }
            },
            {
                id: 'create-account',
                type: 'transform',
                config: {
                    accountId: 'acc-001',
                    status: 'active'
                }
            },
            {
                id: 'send-welcome-email',
                type: 'compute'
            }
        ]
    };

    const input1 = {
        email: 'alice@example.com',
        name: 'Alice Johnson'
    };

    try {
        const start = Date.now();
        const result1 = await engine.execute_workflow_json(
            JSON.stringify(sequenceWorkflow),
            input1
        );
        const elapsed = Date.now() - start;

        console.log('✅ Workflow completed in', elapsed, 'ms');
        console.log('Result:', JSON.stringify(result1, null, 2));
        console.log();
    } catch (error) {
        console.error('❌ Workflow failed:', error.message);
    }

    // Example 2: Parallel workflow
    console.log('Example 2: Parallel Workflow');
    console.log('==============================');

    const parallelWorkflow = {
        id: 'multi-service-validation',
        pattern: 'Parallel',
        tasks: [
            {
                id: 'check-email',
                type: 'validate'
            },
            {
                id: 'check-phone',
                type: 'validate'
            },
            {
                id: 'check-address',
                type: 'validate'
            }
        ]
    };

    const input2 = {
        email: 'bob@example.com',
        phone: '+1234567890',
        address: '123 Main St'
    };

    try {
        const start = Date.now();
        const result2 = await engine.execute_workflow_json(
            JSON.stringify(parallelWorkflow),
            input2
        );
        const elapsed = Date.now() - start;

        console.log('✅ Parallel workflow completed in', elapsed, 'ms');
        console.log('Result:', JSON.stringify(result2, null, 2));
        console.log();
    } catch (error) {
        console.error('❌ Workflow failed:', error.message);
    }

    // Example 3: Choice workflow
    console.log('Example 3: Choice Workflow');
    console.log('==============================');

    const choiceWorkflow = {
        id: 'order-processing',
        pattern: 'Choice',
        tasks: [
            {
                id: 'express-shipping',
                type: 'transform',
                condition: 'express',
                config: {
                    shippingMethod: 'express',
                    estimatedDays: 1
                }
            },
            {
                id: 'standard-shipping',
                type: 'transform',
                condition: 'standard',
                config: {
                    shippingMethod: 'standard',
                    estimatedDays: 5
                }
            }
        ]
    };

    const input3 = {
        express: true,
        orderId: 'order-123'
    };

    try {
        const start = Date.now();
        const result3 = await engine.execute_workflow_json(
            JSON.stringify(choiceWorkflow),
            input3
        );
        const elapsed = Date.now() - start;

        console.log('✅ Choice workflow completed in', elapsed, 'ms');
        console.log('Result:', JSON.stringify(result3, null, 2));
        console.log();
    } catch (error) {
        console.error('❌ Workflow failed:', error.message);
    }

    // Example 4: Validation
    console.log('Example 4: Workflow Validation');
    console.log('================================');

    const invalidWorkflow = {
        id: 'invalid-workflow',
        pattern: 'Sequence'
        // Missing 'tasks' field
    };

    try {
        const valid = engine.validate_workflow(JSON.stringify(invalidWorkflow));
        console.log('✅ Workflow is valid');
    } catch (error) {
        console.log('❌ Validation failed (expected):', error.message);
        console.log();
    }

    // Show statistics
    console.log('Engine Statistics');
    console.log('=================');
    const stats = engine.get_stats();
    console.log('Total Executed:', stats.totalExecuted || 0);
    console.log('Running:', stats.runningWorkflows || 0);
    console.log('Failed:', stats.failedWorkflows || 0);
    console.log('Avg Time:', (stats.avgExecutionTimeMs || 0).toFixed(2), 'ms');
    console.log('Memory Usage:', (stats.memoryUsageBytes || 0) / 1024, 'KB');
    console.log();

    // Benchmark: 100 workflow executions
    console.log('Performance Benchmark');
    console.log('====================');
    console.log('Executing 100 workflows...');

    const benchStart = Date.now();
    for (let i = 0; i < 100; i++) {
        await engine.execute_workflow_json(
            JSON.stringify(sequenceWorkflow),
            input1
        );
    }
    const benchElapsed = Date.now() - benchStart;

    console.log(`✅ 100 workflows executed in ${benchElapsed}ms`);
    console.log(`   Average: ${(benchElapsed / 100).toFixed(2)}ms per workflow`);
    console.log(`   Throughput: ${(100 / (benchElapsed / 1000)).toFixed(2)} workflows/second`);
    console.log();

    // Final statistics
    console.log('Final Statistics');
    console.log('================');
    const finalStats = engine.get_stats();
    console.log('Total Executed:', finalStats.totalExecuted || 0);
    console.log('Avg Time:', (finalStats.avgExecutionTimeMs || 0).toFixed(2), 'ms');
    console.log();

    console.log('✅ All examples completed successfully!');
}

main().catch(console.error);
