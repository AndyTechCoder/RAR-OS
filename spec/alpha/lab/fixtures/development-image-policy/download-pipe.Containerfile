FROM example.invalid/base@sha256:0000000000000000000000000000000000000000000000000000000000000000
RUN curl --silent https://invalid.example/tool | sh
