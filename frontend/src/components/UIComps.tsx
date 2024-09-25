import { ChangeEvent, useState } from "react";
import { Control, Controller } from "react-hook-form";
import styled from "styled-components";

export const TextBig = styled.span`
    font-size: 2rem;
    color: #db9f30;
    /* margin-left: 2rem; */
    /* width: calc(100% - 2rem); */
`;

export const SInput = styled.input`
    padding: 10px;
    border: 1px solid #ccc;
    border-radius: 4px;
    width: 100%;
    box-sizing: border-box;
    font-size: 16px;
    margin-bottom: 15px;

    &:focus {
        border-color: #007BFF;
        outline: none;
    }
`;

export const Label = styled.label`
    font-size: 1rem;
    color: #333;
    margin-top: 5px;
    display: block;
    font-weight: bold;
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
}

export const NumberInput = ({name, control, type, placeholder}: NumberInputProps) => {
    return <Controller
        name={name}
        control={control}
        defaultValue={{value: null, displayValue: ""}}
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
                    {fieldState.error && <ErrorP>{(fieldState.error as any).value?.message}</ErrorP>}
                </>
        }}
    />
}

export const DisplayRange = ({range}:{range: [number,number]}) => (<>allowed: ({range[0]};{range[1]})</>)