ROOT="$(dirname "$(realpath "$0")")"
cd $ROOT/frontend && pnpm run watch &
cd $ROOT/backend && PORT=8080 ADDR=0.0.0.0 DATA_FOLDER=${ROOT}/data PASSWORD=dev STATIC_DIR=../static cargo watch -x run