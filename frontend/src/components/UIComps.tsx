import { ChangeEvent, useState } from "react";
import { Control, Controller } from "react-hook-form";
import styled from "styled-components";

export const TextBig = styled.div`
    font-size: 2rem;
    color: #db9f30;
    /* margin-left: 2rem; */
    /* width: calc(100% - 2rem); */
`;

export const SInput = styled.input`
    margin-bottom: 0.5rem;
    width: 17rem;
`;

export const ErrorP = styled.p`
    color: #c01111;
    font-size: 0.75rem;
`;

type NumberInputProps = {
    control: Control<any>,
    name: string,
    type: "float" | "int",
    placeholder: string,
    opts?: {
        min?: number,
        max?: number
    }
}

export const NumberInput = ({name, control, type, placeholder, opts}: NumberInputProps) => {
    return <Controller
        name={name}
        control={control}
        defaultValue={{value: null, displayValue: ""}}
        rules={{
            validate: (value) => {
                // If opts are provided, validate min and max
                const { value: numericValue } = value;
                if (opts?.min !== undefined && numericValue < opts.min) {
                    return `Value must be greater than or equal to ${opts.min}`;
                }
                if (opts?.max !== undefined && numericValue > opts.max) {
                    return `Value must be less than or equal to ${opts.max}`;
                }
                return true;
            },
        }}
        render={({
                field,
                fieldState
            }) => {
                const onChange = (ev: ChangeEvent<HTMLInputElement>) => {
                    const { value } = ev.target;

                    let parsedValue: number | null = null;

                    // Parse based on type (int or float)
                    if (type === "float") {
                        parsedValue = parseFloat(value);
                    } else if (type === "int") {
                        parsedValue = parseInt(value);
                    }

                    // Handle NaN and empty values
                    if (!Number.isNaN(parsedValue) && value !== "") {
                        field.onChange({value: parsedValue, displayValue: value});
                    } else {
                        // Handle empty input case
                        field.onChange({value: null, displayValue: ""});
                    }
                };

                return <>
                    <SInput ref={field.ref} value={field.value.displayValue} onChange={onChange} placeholder={placeholder} />
                    {fieldState.error && <ErrorP>{fieldState.error.message}</ErrorP>}
                </>
        }}
    />
}