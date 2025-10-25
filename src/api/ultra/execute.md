# null

## OpenAPI

````yaml openapi-spec/ultra/ultra.yaml post /execute
paths:
  path: /execute
  method: post
  servers:
    - url: https://lite-api.jup.ag/ultra/v1
      description: Free tier API endpoint with rate limits
    - url: https://api.jup.ag/ultra/v1
      description: >-
        Paid tier API endpoint with higher rate limits to be used with an API
        Key
  request:
    security: []
    parameters:
      path: {}
      query: {}
      header: {}
      cookie: {}
    body:
      application/json:
        schemaArray:
          - type: object
            properties:
              signedTransaction:
                allOf:
                  - type: string
                    description: |
                      - The signed transaction to execute
              requestId:
                allOf:
                  - type: string
                    description: |
                      - Found in response of `/order`
            requiredProperties:
              - signedTransaction
              - requestId
        examples:
          example:
            value:
              signedTransaction: <string>
              requestId: <string>
  response:
    '200':
      application/json:
        schemaArray:
          - type: object
            properties:
              status:
                allOf:
                  - type: string
                    enum:
                      - Success
                      - Failed
              signature:
                allOf:
                  - type: string
              slot:
                allOf:
                  - type: string
              error:
                allOf:
                  - type: string
              code:
                allOf:
                  - type: number
              totalInputAmount:
                allOf:
                  - type: string
              totalOutputAmount:
                allOf:
                  - type: string
              inputAmountResult:
                allOf:
                  - type: string
              outputAmountResult:
                allOf:
                  - type: string
              swapEvents:
                allOf:
                  - type: array
                    items:
                      type: object
                      properties:
                        inputMint:
                          type: string
                        inputAmount:
                          type: string
                        outputMint:
                          type: string
                        outputAmount:
                          type: string
                      required:
                        - inputMint
                        - inputAmount
                        - outputMint
                        - outputAmount
            requiredProperties:
              - status
              - code
        examples:
          example:
            value:
              status: Success
              signature: <string>
              slot: <string>
              error: <string>
              code: 123
              totalInputAmount: <string>
              totalOutputAmount: <string>
              inputAmountResult: <string>
              outputAmountResult: <string>
              swapEvents:
                - inputMint: <string>
                  inputAmount: <string>
                  outputMint: <string>
                  outputAmount: <string>
        description: Successful response
    '400':
      application/json:
        schemaArray:
          - type: object
            properties:
              error:
                allOf:
                  - type: string
              code:
                allOf:
                  - type: number
            requiredProperties:
              - error
              - code
        examples:
          example:
            value:
              error: <string>
              code: 123
        description: Bad request
    '500':
      application/json:
        schemaArray:
          - type: object
            properties:
              error:
                allOf:
                  - type: string
              code:
                allOf:
                  - type: number
            requiredProperties:
              - error
              - code
        examples:
          example:
            value:
              error: <string>
              code: 123
        description: Internal server error
  deprecated: false
  type: path
components:
  schemas: {}

````