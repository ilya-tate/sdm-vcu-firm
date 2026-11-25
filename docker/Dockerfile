FROM alpine:3.22

RUN --mount=type=cache,target=/etc/apk/cache \
    apk add python3 py3-elftools qemu-system-arm

COPY run_tests.py /

ENTRYPOINT ["python3", "/run_tests.py"]
