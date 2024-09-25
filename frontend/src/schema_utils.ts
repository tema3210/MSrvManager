export const NumberInputData = (minimum: number,maximum: number) => ({
    type: "object",
    properties: {
      value: { 
        type: "number",
        minimum,
        maximum,
        errorMessage: {
          type: "Value must be a valid number within range",
          minimum: `Minimum value is ${minimum}`,
          maximum: `Maximum value is ${maximum}`
        }
      },
      displayValue: { type: "string" }
    },
    required: ["value"],
});