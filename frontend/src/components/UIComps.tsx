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
}

export const NumberInput = ({name, control, type, placeholder}: NumberInputProps) => {
    // <SInput type="number" step="0.2" {...register("maxMemory")} placeholder="max allowed memory consumption" /><br />
    return <Controller
        name={name}
        control={control}
        render={({
                field,
                fieldState
            }) => {
                const onChange = (ev: ChangeEvent<HTMLInputElement>) => {
                    const { value } = ev.target;

                    // Check if value ends with a dot
                    const endsWithDot = value.endsWith(".");

                    let parsedValue: number | null = null;

                    // Parse based on type (int or float)
                    if (type === "float") {
                        parsedValue = parseFloat(value);
                    } else if (type === "int") {
                        parsedValue = parseInt(value);
                    }

                    // Handle NaN and empty values
                    if (!Number.isNaN(parsedValue) && value !== "") {
                        const finalValue = endsWithDot ? value : parsedValue;
                        field.onChange(finalValue);
                    } else {
                        // Handle empty input case
                        field.onChange(null);
                    }
                };

                return <>
                    <SInput ref={field.ref} value={field.value} onChange={onChange} placeholder={placeholder} />
                    {fieldState.error && <ErrorP>{fieldState.error.message}</ErrorP>}
                </>
        }}
    />
}