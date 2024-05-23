# Call multiple steps piping

```graphql @config
schema {
  query: Query
}

type Query {
  a_input(input: JSON): JSON @expr(body: {input: "{{.args.input.a}}"})
  b_input(input: JSON): JSON @expr(body: {input: "{{.args.input.b}}"})
  a(input: JSON): JSON @expr(body: "{{.args.input.a}}")
  b(input: JSON): JSON @expr(body: "{{.args.input.b}}")
  c(input: JSON): JSON @expr(body: "{{.args.input.c}}")
  wrap_args: JSON @expr(body: {input: "{{.args}}"})
  wrap_input(input: JSON): JSON @expr(body: {input: "{{.args.input}}"})

  abc_input(input: JSON): JSON
    @call(
      steps: [
        {query: "wrapInput", args: {input: "{{.args.input}}"}}
        {query: "aInput"}
        {query: "wrapInput"}
        {query: "bInput"}
        {query: "wrapInput"}
        {query: "c"}
      ]
    )
  abc(input: JSON): JSON
    @call(
      steps: [
        {query: "a", args: {input: "{{.args.input}}"}}
        {query: "wrapArgs"}
        {query: "b"}
        {query: "wrapArgs"}
        {query: "c"}
      ]
    )
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc_input(input: {a: {b: {c: 3}}})}"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc(input: {a: {b: {c: 3}}}) }"
```
