FROM example.invalid/base@sha256:0000000000000000000000000000000000000000000000000000000000000000
RUN wget -qO- https://invalid.example/tool | env sh
