curl -X POST http://localhost:8080/session -v -4 -i \
     -H 'Content-Type: application/json' \
     -H 'Accept: application/json' \
     -d '{"username":"alice", "password": "secret123"}'
