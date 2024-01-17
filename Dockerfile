# Build the manager binary
FROM golang:1.21 as builder

LABEL org.opencontainers.image.source=https://github.com/kubernetes-sigs/blixt
LABEL org.opencontainers.image.description="An experimental layer 4 load-balancer built using eBPF/XDP with ebpf-go \
for use in Kubernetes via the Kubernetes Gateway API"
LABEL org.opencontainers.image.licenses=Apache-2.0

WORKDIR /workspace
# Copy the Go Modules manifests
COPY go.mod go.mod
COPY go.sum go.sum
# cache deps before building and copying source so that we don't need to re-download as much
# and so that source changes don't invalidate our downloaded layer
RUN go mod download

# Copy the go source
COPY main.go main.go
COPY controllers/ controllers/
COPY pkg/ pkg/
COPY internal/ internal/

# Build Delve
RUN CGO_ENABLED=0 go install -ldflags "-s -w -extldflags '-static'" github.com/go-delve/delve/cmd/dlv@latest
# Build
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -gcflags "all=-N -l" -a -o manager main.go

# Use distroless as minimal base image to package the manager binary
# Refer to https://github.com/GoogleContainerTools/distroless for more details
FROM ubuntu:22.04
WORKDIR /
COPY --from=builder /workspace/manager .
COPY LICENSE /workspace/LICENSE

USER root

RUN apt-get update && \
    apt-get install -y net-tools procps && \
    rm -rf /var/lib/apt/lists/*

# USER 65532:65532

# if dlv release new version, please update this
COPY --from=builder /go/bin/dlv /dlv

ENTRYPOINT ["/dlv", "--listen=:40000", "--headless=true", "--api-version=2", "--accept-multiclient", "exec", "/manager"]

EXPOSE 40000
