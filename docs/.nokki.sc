val nokkiOptions = options {

}

val nokkiPipeline = pipeline {
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

    job("deploy_to_stage") {
        inStage("deploy")
        script {
            - "echo 'Starting deploy..."
            - "scripts/deploy.sh"
            - "helm upgrade --install"
                "chart-path release_name --option one --option two"
        }
    }
}