var nokkiIfPipeline = pipeline {
    stage("build")
    stage("test")
    stage("deploy")

    job("compile_code") {
        inStage("build")
        image("rust:1.75")
        run("cargo build --release")
        saveArtifacts("target/release/nokki")
    }

    job("unit_tests") {
        inStage("test")
        withMatrix("os" -> List("ubuntu-latest", "macos-latest"))
        run(s"echo Running tests on ${os}")
        run("cargo test")
    }

    job("deploy") {
        inStage("deploy")

        rules {
          - if ${tag} == "v*" {
                when = manual
            }    
          - if ${sourceBranch} == "main" 
          - if ${ciPipelineSource} == "schedule" {
                when = never
            }   
        }

        run("./deploy.sh")
    }
}