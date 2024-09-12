ROOT="$(dirname "$(realpath "$0")")"
cd $ROOT/frontend && npm run build
cd $ROOT/backend && cargo build --release
if [[ $? -eq 0 ]] then
TARGET_SUBFOLDER="x86_64-unknown-linux-musl/release"
# TARGET_SUBFOLDER="release"
cp $ROOT/backend/target/$TARGET_SUBFOLDER/msrvmanager $ROOT/msrvmanager
docker build --target image -f $ROOT/Dockerfile -t msrvmanager:latest $ROOT
fi

