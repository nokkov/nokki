nokki_pipeline! {
    metadata {
        name: "Enterprise Rust Pipeline",
        version: "1.2.0",
        author: "Nokki Dev Team"
    }

    using: {
        nokki_std::rust as rust,
        nokki_std::docker as docker,
        "custom-registry/devops-lib" as devops
    };

    default {
        image: "rust:1.75-slim",
        timeout: "30m",
        tags: ["shared-runner"],
        after_script: ["echo 'Job stage finished'"]
    }

    stages: "build", "test", "deploy";

    job "compile" => {
        stage: "build";
        image: "rust:1.75";
        tags: ["high-performance"];
        timeout: "20m";

        variables: {
            "CARGO_INCREMENTAL": "0",
            "RUST_BACKTRACE": "full"
        };

        cache: {
            key: "rust-cache-{{ hash 'Cargo.lock' }}",
            paths: [
                "target",
                "~/.cargo/registry",
                "~/.cargo/git"
            ],
            policy: "pull-push",
            fallback_key: "rust-cache-master"
        };

        artifacts: {
            name: "nokki-binary-${{ CI_COMMIT_SHORT_SHA }}",
            paths: ["target/release/nokki"],
            exclude: ["target/release/*.d"],
            expire_in: "30 days",
            public: false
        };

        before_script: [
            "rustup component add clippy",
            "echo 'Starting build for commit ${{ CI_COMMIT_SHA }}'"
        ];

        script: {
            rust::cargo_build(release: true);
        };

        after_script: [
            "ls -lh target/release/",
            "echo 'Cleaning up temporary files...'"
        ];

        on_success: {
            run: "echo 'Build successful! Artifacts are ready.'";
        };

        on_failure: {
            run: "echo 'Build failed. Check clippy logs.'";
        };

        on_abort: {
            run: "echo 'Job was cancelled by user. Rolling back...'";
        };
    }

    job "multi_platform_test" => {
        stage: "test";
        needs: ["compile"];
        
        matrix: {
            "os": ["ubuntu-latest", "macos-latest"]
        };

        // sidecar services
        services: {
            "db": image("postgres:15-alpine")
                .env("POSTGRES_PASSWORD", "pass")
                .env("POSTGRES_DB", "test_db"),
            "redis": image("redis:7-alpine")
        };

        tags: ["${{ matrix.os }}"];
        
        cache: {
            key: "rust-cache-{{ hash 'Cargo.lock' }}",
            paths: ["target"],
            policy: "pull-only"
        };

        run: "cargo test".retry(2);

        on_failure: {
            run: "notify-slack --channel #dev-alerts --msg 'Tests failed on ${{ matrix.os }}'";
        };
    }

    job "canary_deploy" => {
        stage: "deploy";

        rules: [
            if branch == "main",
            if source != "schedule",
            if tag.matches("v*") => { when: manual }
        ];
        
        input: {
            param "traffic" => {
                default: 10,
                description: "Percentage of traffic"
            }
        };

        environment: {
            name: "canary",
            url: "https://canary-${{ CI_COMMIT_SHORT_SHA }}.nokki.dev",
        };

        secrets: {
            // Подтягиваем из встроенного хранилища Nokki/Gitea
            "KUBE_TOKEN": secret("PROD_KUBE_TOKEN"),
            
            // Подтягиваем напрямую из HashiCorp Vault
            "DB_PASSWORD": vault {
                path: "secret/data/nokki/prod/db",
                field: "password",
                engine: "v2"
            }
        };

        run: "helm upgrade --install --set traffic=${{ traffic }}";

        on_success: {
            run: "echo 'Deployment to Canary finished successfully'";
        };
    }
}
