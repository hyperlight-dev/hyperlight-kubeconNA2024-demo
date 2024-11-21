wget https://stable.release.flatcar-linux.net/amd64-usr/current/flatcar_production_qemu.sh

chmod +x flatcar_production_qemu.sh

wget https://stable.release.flatcar-linux.net/amd64-usr/current/flatcar_production_qemu_image.img

./flatcar_production_qemu.sh -curses

git clone https://github.com/hyperlight-dev/hyperlight-kubeconNA2024-demo

git checkout danbugs/flatcar

./flatcar/demo-main &

curl http://127.0.0.1:3030/hyperlight/hello-world/cold
