match engine.get_case(case_id).await {
    Ok(case) => println!("Case found: {:?}", case),
    Err(e) => println!("Case not found: {}", e),
}