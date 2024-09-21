import { ChangeEvent } from "react";
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
                    let parsed = null;
                    switch (type) {
                        case "float":
                            parsed = parseFloat(ev.target.value);
                            break;
                        case "int":
                            parsed = parseInt(ev.target.value);
                            break;
                    }
                    if (ev.target.value === "") {
                        return field.onChange(null)
                    }
                    if (parsed && !Number.isNaN(parsed) ) {
                        return field.onChange(parsed)
                    }
                }

                return <>
                    <SInput value={field.value} onChange={onChange} placeholder={placeholder} />
                    {fieldState.error && <ErrorP>{fieldState.error.message}</ErrorP>}
                </>
        }}
    />
}