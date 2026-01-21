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

        if(exist(./tests))

        withMatrix("os" -> List("ubuntu-latest", "macos-latest"))
        run(s"echo Running tests on ${os}")
        run("cargo test")
    }

    job("deploy") {
        inStage("deploy")
        
        manualIf(branch == "main" && tag ~= "v.*")
        neverIf(source == "schedule" || env == "test")

        run("./deploy.sh")
    }
}