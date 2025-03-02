# 电梯电瓶车报警

## 功能概述
镜头前出现电瓶车立即报警，不会对同一辆车重复报警。本服务和[猫猫围栏](../cat_finder/README.md)在推送时机上有区别，猫是离开时才通知，电瓶车是进入就提示。一些安装命令和常见问题处理方式在[猫猫围栏](../cat_finder/README.md)已经说明，强烈建议先看上一个教程。

## 软硬件环境

*nix 系统（Linux/Mac），x86 芯片。支持 onnx runtime 即可。

## [模型下载](../../../docs/download-models.zh.md)

## 启动服务

安装运行依赖
```bash
$ apt install redis-server
$ redis-server &
...
$ conda activate py38
$ pip3 install onnxruntime --user
```

准备一个 rtsp 视频流地址或者视频文件绝对路径做测试输入。相关教程已整合在 [如何生成自己的 rtsp 流地址](../../../docs/how-to-build-and-run/generate-rtsp.zh.md) 。这里仅仅需要
```bash
$ wget https://github.com/aler9/rtsp-simple-server/releases/download/v0.17.2/rtsp-simple-server_v0.17.2_linux_amd64.tar.gz
$ 
$ tar xvf rtsp-simple-server_v0.17.2_linux_amd64.tar.gz && ./rtsp-simple-server 
$ ffmpeg -re -stream_loop -1 -i ${models}/cat_finder_testdata/test1.ts -c copy -f rtsp rtsp://127.0.0.1:8554/test1.ts
```

启动服务
```bash
$ cd flow-python/examples
$ run_with_plugins -c electric_bicycle/electric_bicycle.toml  -p electric_bicycle
```
服务配置文件在`electric_bicycle/electric_bicycle.toml`，解释参考 [how-to-add-graph](../../../docs/how-to-add-my-service/01-single-classification-model.zh.md) 。这里只需要打开 8083 端口服务，操作和[猫猫围栏](../cat_finder/README.md) 近似。

```bash
$ google-chrome-stable  http://127.0.0.1:8083/docs 
```
