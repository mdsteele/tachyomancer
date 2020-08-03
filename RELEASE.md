# Release Process

## Tachyoscope

From the repository root, run:

```shell
# Make sure the tests pass:
cargo test -p tachy -p tachyoscope

# Build the container:
docker build -t tachyoscope-appengine -f ./tachyoscope/Dockerfile .

# Make sure the container works:
CONTAINER_ID=`docker run -d -p 8081:8080 tachyoscope-appengine`
curl http://localhost:8081/readiness_check  # Should print "ready"
docker stop ${CONTAINER_ID}

# TODO: Push to App Engine
```
