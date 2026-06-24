hard-reset-db:
    docker-compose down -v && docker-compose up -d && sleep 2 && sqlx migrate run

test:
    cargo test -- --nocapture

ex-llm:
    cargo run --example llm_recommend

ex-tmdb query="Breaking Bad":
    cargo run --example tmdb_search -- "{{query}}"

ex-db:
    cargo run --example db_flows

ex:
    echo "Running all examples"
    echo ""
    echo "> Running llm examples"
    just ex-llm
    echo ""
    echo "> Running tmdb examples"
    just ex-tmdb
    echo ""
    echo "> Running db examples"
    just ex-db

reset-db:
    cargo sqlx database reset --force -y
