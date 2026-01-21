val nokkiPipeline = pipeline {
    stages("build", "test", "deploy")

    job("compile_code") {
        stage("build")
        image("rust:1.75")
                
        artifacts { path("target/release/nokki") }
        
        run("cargo build --release")
    }

    job("unit_tests") {
        stage("test")
        needs("compile_code")
        
        allowFailure(true)
        retry(3)
        
        run("cargo test")
    }

    job("canary_deploy") {
        stage("deploy")
        
        input {
            val percent = param("traffic", default = "10%")
        }

        script {
            if (percent != "0%") {
                run(s"helm upgrade --set traffic=${percent}")
            }
        }
    }
}