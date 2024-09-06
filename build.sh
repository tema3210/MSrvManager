ROOT="$(dirname "$(realpath "$0")")"
cd $ROOT/frontend && npm run build
cd $ROOT/backend && cargo build --release
if [[ $? -eq 0 ]] then
cp $ROOT/backend/target/release/msrvmanager $ROOT/msrvmanager
docker build --target image -f $ROOT/Dockerfile -t msrvmanager:latest $ROOT
fi

