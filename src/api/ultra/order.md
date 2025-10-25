# null

## OpenAPI

````yaml openapi-spec/ultra/ultra.yaml get /order
paths:
  path: /order
  method: get
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
      query:
        inputMint:
          schema:
            - type: string
              required: true
        outputMint:
          schema:
            - type: string
              required: true
        amount:
          schema:
            - type: string
              required: true
        taker:
          schema:
            - type: string
              required: false
        referralAccount:
          schema:
            - type: string
              required: false
              description: |
                - Refer to [Integrator Fees](/docs/ultra/fee) for more details
        referralFee:
          schema:
            - type: number
              required: false
              description: |
                - Refer to [Integrator Fees](/docs/ultra/fee) for more details
              maximum: 255
              minimum: 50
        excludeRouters:
          schema:
            - type: enum<string>
              enum:
                - metis
                - jupiterz
                - dflow
                - okx
              required: false
        excludeDexes:
          schema:
            - type: string
              description: >
                - [Full list of DEXes
                here](https://lite-api.jup.ag/swap/v1/program-id-to-label), for
                example: `excludeDexes=Raydium,Orca+V2,Meteora+DLMM`

                - **Important**: This only excludes DEXes on the Metis router,
                does not apply to other routers

                - For example:
                  - **Exclude** Raydium: `excludeRouters=<all-except-Metis>` and `excludeDexes=Raydium`
                  - **Only include** Meteora DLMM: `excludeRouters=<all-except-Metis>` and `excludeDexes=<all-except-MeteoraDLMM>`
        payer:
          schema:
            - type: string
              required: false
              description: >
                - The address of an external gas payer to pay for the network
                fees and rent on behalf of the taker

                - Refer to [Integrator Gas Payer](/docs/ultra/payer) for more
                details
      header: {}
      cookie: {}
    body: {}
  response:
    '200':
      application/json:
        schemaArray:
          - type: object
            properties:
              mode:
                allOf:
                  - type: string
              inputMint:
                allOf:
                  - type: string
              outputMint:
                allOf:
                  - type: string
              inAmount:
                allOf:
                  - type: string
              outAmount:
                allOf:
                  - type: string
              otherAmountThreshold:
                allOf:
                  - type: string
              swapMode:
                allOf:
                  - type: string
              slippageBps:
                allOf:
                  - type: number
              inUsdValue:
                allOf:
                  - type: number
              outUsdValue:
                allOf:
                  - type: number
              priceImpact:
                allOf:
                  - type: number
              swapUsdValue:
                allOf:
                  - type: number
              priceImpactPct:
                allOf:
                  - type: string
                    description: >
                      - Please use `priceImpact` field instead, this is still
                      available only for backwards compatibility
              routePlan:
                allOf:
                  - type: array
                    items:
                      type: object
                      properties:
                        swapInfo:
                          type: object
                          properties:
                            ammKey:
                              type: string
                            label:
                              type: string
                            inputMint:
                              type: string
                            outputMint:
                              type: string
                            inAmount:
                              type: string
                            outAmount:
                              type: string
                            feeAmount:
                              type: string
                            feeMint:
                              type: string
                          required:
                            - ammKey
                            - label
                            - inputMint
                            - outputMint
                            - inAmount
                            - outAmount
                            - feeAmount
                            - feeMint
                        percent:
                          type: number
                        bps:
                          type: number
                      required:
                        - swapInfo
                        - percent
                        - bps
              feeMint:
                allOf:
                  - type: string
              feeBps:
                allOf:
                  - type: number
              signatureFeeLamports:
                allOf:
                  - type: number
                    description: >
                      - The number of lamports the taker has to pay as a base
                      network fee, if a valid transaction is returned. This may
                      be 0 if the transaction is gasless, in which case the gas
                      payer will cover this fee.
              prioritizationFeeLamports:
                allOf:
                  - type: number
                    description: >
                      - The number of lamports the taker has to pay for higher
                      priority landing, if a valid transaction is returned.
                      Includes priority fees and tips for services such as Jito,
                      if any. This may be 0 if the transaction is gasless, in
                      which case the gas payer will cover this fee.
              rentFeeLamports:
                allOf:
                  - type: number
                    description: >
                      - The number of lamports the taker has to pay for rent, if
                      a valid transaction is returned. This may be 0 if the
                      transaction is gasless, in which case the gas payer will
                      cover this fee. Note that this value is just an estimate.
              swapType:
                allOf:
                  - type: string
                    description: |
                      - Deprecated, in favour of router
              router:
                allOf:
                  - type: string
                    enum:
                      - aggregator
                      - jupiterz
                      - dflow
                      - okx
              transaction:
                allOf:
                  - type: string
                    nullable: true
                    description: >
                      - Unsigned base-64 encoded transaction to be signed and
                      used in `/execute`

                      - If `taker` is null, this field will be null. Else, it
                      will either be a valid base64 encoded transaction or the
                      empty string
              gasless:
                allOf:
                  - type: boolean
              requestId:
                allOf:
                  - description: |
                      - Required to make a request to `/execute`
                    type: string
              totalTime:
                allOf:
                  - type: number
              taker:
                allOf:
                  - type: string
                    nullable: true
              quoteId:
                allOf:
                  - type: string
              maker:
                allOf:
                  - type: string
              expireAt:
                allOf:
                  - type: string
              platformFee:
                allOf:
                  - type: object
                    properties:
                      amount:
                        type: string
                      feeBps:
                        type: number
                    required:
                      - amount
                      - feeBps
              errorCode:
                allOf:
                  - type: number
                    enum:
                      - 1
                      - 2
                      - 3
                    description: >
                      - This field will be present if `taker` is defined and
                      `transaction` is the empty string

                      - It is unique for each error scenarios
              errorMessage:
                allOf:
                  - type: string
                    enum:
                      - Insufficient funds
                      - Top up `${solAmount}` SOL for gas
                      - Minimum `${swapAmount}` for gasless
                    description: >
                      - This field will be present if `taker` is defined and
                      `transaction` is the empty string

                      - This field can still return despite having a valid
                      order/quote

                      - This is meant for display purposes only and it is
                      discouraged to match these error messages as they could be
                      parameterized
            requiredProperties:
              - mode
              - inputMint
              - outputMint
              - inAmount
              - outAmount
              - otherAmountThreshold
              - priceImpactPct
              - swapMode
              - slippageBps
              - routePlan
              - feeBps
              - signatureFeeLamports
              - prioritizationFeeLamports
              - rentFeeLamports
              - swapType
              - router
              - transaction
              - gasless
              - requestId
              - totalTime
              - taker
        examples:
          example:
            value:
              mode: <string>
              inputMint: <string>
              outputMint: <string>
              inAmount: <string>
              outAmount: <string>
              otherAmountThreshold: <string>
              swapMode: <string>
              slippageBps: 123
              inUsdValue: 123
              outUsdValue: 123
              priceImpact: 123
              swapUsdValue: 123
              priceImpactPct: <string>
              routePlan:
                - swapInfo:
                    ammKey: <string>
                    label: <string>
                    inputMint: <string>
                    outputMint: <string>
                    inAmount: <string>
                    outAmount: <string>
                    feeAmount: <string>
                    feeMint: <string>
                  percent: 123
                  bps: 123
              feeMint: <string>
              feeBps: 123
              signatureFeeLamports: 123
              prioritizationFeeLamports: 123
              rentFeeLamports: 123
              swapType: <string>
              router: aggregator
              transaction: <string>
              gasless: true
              requestId: <string>
              totalTime: 123
              taker: <string>
              quoteId: <string>
              maker: <string>
              expireAt: <string>
              platformFee:
                amount: <string>
                feeBps: 123
              errorCode: 1
              errorMessage: Insufficient funds
        description: Successful response
    '400':
      application/json:
        schemaArray:
          - type: object
            properties:
              error:
                allOf:
                  - type: string
            requiredProperties:
              - error
        examples:
          example:
            value:
              error: <string>
        description: Bad request
    '500':
      application/json:
        schemaArray:
          - type: object
            properties:
              error:
                allOf:
                  - type: string
            requiredProperties:
              - error
        examples:
          example:
            value:
              error: <string>
        description: Internal server error
  deprecated: false
  type: path
components:
  schemas: {}

````