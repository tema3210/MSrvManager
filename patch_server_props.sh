#!/bin/bash

sed -i "s/^server-port=[0-9]\+/server-port=$MPORT/" "$PROPERTIES_FILE"

sed -i "s/^rcon.port=[0-9]\+/rcon.port=$MRCON/" "$PROPERTIES_FILE"

sed -i "s/^enable-rcon=.*/enable-rcon=true/" "$PROPERTIES_FILE"

sed -i "s/^rcon.password=.*/rcon.password=$PASSWORD/" "$PROPERTIES_FILE"

sed -i "s/^online-mode=.*/online-mode=false/" "$PROPERTIES_FILE"
