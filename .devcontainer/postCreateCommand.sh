#! /bin/sh

sudo apt update -y &&\
sudo apt upgrade -y &&\
sudo apt install -y libsdl2-dev &&\
pip install -r requirements.txt
