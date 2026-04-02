#!/bin/bash

# static/nsjail/entrypoint.sh
# Notes:
# - Creates initial cgroup and migrate all processes to alter cgroup subtree control.
# - Creates nsjail cgroup.
# - Runs the entrypoint.

# create cgroup
echo "Initializing initial cgroup"
mkdir -p /sys/fs/cgroup/init
xargs -rn1 < /sys/fs/cgroup/cgroup.procs > /sys/fs/cgroup/init/cgroup.procs
echo "+cpu +cpuset +memory +pids" > /sys/fs/cgroup/cgroup.subtree_control

echo "Initializing isolate cgroup"
mkdir -p /sys/fs/cgroup/nsjail
echo "+cpu +cpuset +memory +pids" > /sys/fs/cgroup/nsjail/cgroup.subtree_control


echo "Done"

exec "$@"