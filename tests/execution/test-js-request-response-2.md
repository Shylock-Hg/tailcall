# Js Request Response Hello World

This test is just a mirror of existing `test-js-request-response.md` but with changed values. It exists to test that js runtime is created separately for every app_ctx and they not interfere with each other.

```js @file:test.js
function onRequest({request}) {
  if (request.uri.path.endsWith("/hello")) {
    return {
      response: {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify("darkness"),
      },
    }
  } else if (request.uri.path.endsWith("/hi")) {
    request.uri.path = "/old-friend"
    console.log({request})
    return {request}
  } else {
    return {request}
  }
}
```

```yml @config
upstream:
  onRequest: "onRequest"
links:
  - type: Script
    src: "test.js"
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:3000/hello")
  hi: String @http(url: "http://localhost:3000/hi")
}
```

```yml @mock
- request:
    method: GET
    url: http://localhost:3000/old-friend
  response:
    status: 200
    body: I've come to talk with you again
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { hello hi }
```
