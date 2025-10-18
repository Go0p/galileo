# program-id-to-label

> Returns a hash, which key is the program id and value is the label.
This is used to help map error from transaction by identifying the fault program id.
This can be used in conjunction with the `excludeDexes` or `dexes` parameter.


## OpenAPI

````yaml openapi-spec/swap/swap.yaml get /program-id-to-label
paths:
  path: /program-id-to-label
  method: get
  servers:
    - url: https://lite-api.jup.ag/swap/v1
      description: Free tier API endpoint with rate limits
    - url: https://api.jup.ag/swap/v1
      description: >-
        Paid tier API endpoint with higher rate limits to be used with an API
        Key
    - url: https://preprod-quote-api.jup.ag/
      description: This is a staging endpoint for tests
  request:
    security: []
    parameters:
      path: {}
      query: {}
      header: {}
      cookie: {}
    body: {}
  response:
    '200':
      application/json:
        schemaArray:
          - type: object
            properties: {}
            additionalProperties:
              allOf:
                - type: string
        examples:
          example:
            value: {}
        description: Default response
  deprecated: false
  type: path
components:
  schemas: {}

````