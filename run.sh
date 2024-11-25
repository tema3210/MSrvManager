ROOT="$(dirname "$(realpath "$0")")"

cd $ROOT/frontend && pnpm run watch &
cd $ROOT/backend && cargo watch -s "$ROOT/backend/start_dev.sh"