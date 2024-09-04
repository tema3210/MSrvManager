ROOT="$(dirname "$(realpath "$0")")"
cd $ROOT/frontend && npm run build
cd $ROOT/backend && cargo build --release
if [[ $? -eq 0 ]] then
cp $ROOT/backend/target/release/graph_ql_test $ROOT/graph_ql_test
docker build --target image -f $ROOT/Dockerfile -t msrvmanager:latest $ROOT
fi

