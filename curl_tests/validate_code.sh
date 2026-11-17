curl -X POST http://localhost:8080/session/verification -v -4 -i \
     -H 'Content-Type: application/json' \
     -H 'Accept: application/json' \
     -d "{\"request_id\": \"${REQUEST_ID}\", \"username\": \"alice\", \"code\": \"$CODE\"}"
