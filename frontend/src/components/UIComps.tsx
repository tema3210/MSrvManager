import { ChangeEvent } from "react";
import { Control, Controller } from "react-hook-form";
import styled from "styled-components";

export const TextBig = styled.span`
    font-size: 1.7rem;
    color: #db9f30;
    /* margin-left: 2rem; */
    /* width: calc(100% - 2rem); */
`;


export const TextSmall = styled.span`
    font-size: 1.2rem;
    color: black;
    /* margin-left: 2rem; */
    /* width: calc(100% - 2rem); */
`;

export const BrokenServerLink = styled.a`
    text-decoration: none;
    color: #000000;

    margin-right: 1rem;

    &:hover {
        text-decoration: underline;
    }
`;

export const HomeLink = styled.a`
    text-decoration: none;
    color: #db9f30;

    font-size: 2rem;
    margin-right: 1rem;

    &:hover {
        text-decoration: underline;
    }
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
        border-color: #db9f30;
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
    type: "float" | "int"
}

type HookFormProps = {
    control: Control<any>,
    name: string,
    placeholder?: any
};

export const NumberInput = ({name, control, type, placeholder}: NumberInputProps & HookFormProps) => {
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

export const DisplayRange = ({range}:{range: [number,number]}) => (<>allowed: {`[${range[0]};${range[1]}]`}</>)

type InstanceStateDisplayProps = {
    serverName: string;
    state: string;
};

const InstanceStateContainer = styled.div`
    margin: 10px 0;
    padding: 10px;
    border: 1px solid #ccc;
    border-radius: 4px;
`;

const ServerName = styled.span`
    font-weight: bold;
    color: #333;
`;

const State = styled.span`
    color: #db9f30;
`;

export const InstanceStateDisplay = ({ serverName, state }: InstanceStateDisplayProps) => {
    return (
        <InstanceStateContainer>
            <ServerName>{serverName}</ServerName>: <State>{state}</State>
        </InstanceStateContainer>
    );
};

const TextAreaInner = styled.textarea<{ height: number }>`
    width: 100%;
    height: ${({ height }) => `${(height + 1) * 1.2}rem`};
    line-height: 1.2rem;
    padding: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    box-sizing: border-box;
    font-size: 1rem;
    margin-bottom: 15px;
    resize: none;

    &:focus {
        border-color: #db9f30;
        outline: none;
    }
`;

type TextAreaProps = {};

export const TextArea = ({name, control, placeholder}: TextAreaProps & HookFormProps) => (
    <Controller
        name={name}
        control={control}
        defaultValue=""
        render={({field}) => {
            const height = (field.value ?? "").split("\n").length + 1;
            return <TextAreaInner {...field} height={height} placeholder={placeholder} />
        }}
    />
)