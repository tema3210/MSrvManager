import Spinner from "./components/Spinner";
import { makeOnLoad, SSRProps } from "./lib";
import { useQuery, gql, useMutation, useSubscription } from '@apollo/client';
import { InstanceDescriptor } from "./model";
import { useForm } from "react-hook-form";
import { ajvResolver } from "@hookform/resolvers/ajv";
import { fullFormats } from "ajv-formats/dist/formats";
import { DisplayRange, ErrorP, HomeLink, Label, NumberInput, SInput, TextBig } from "./components/UIComps";
import Btn from "./components/Button";
import { NumberInputData } from "./schema_utils";
import { useMemo } from "react";

type FormData = {
    maxMemory: {value: number, displayValue: string};
    port: {value: number, displayValue: string};
};

const Alter = ({pageData}: SSRProps) => {

    const { data: ports } = useQuery<{ portsTaken: {portLimits: [number,number]} }>(gql`
        {
            portsTaken {
                portLimits,
            }
        }
    `);

    const { loading, error, data } = useSubscription<{instance: InstanceDescriptor | null}>(
        gql`
            subscription Subscription($name: String!) {
                instance(name: $name)
            }
        `, 
        {
            variables: {
                name: pageData.name
            }
        }
    );

    const instanceData = data?.instance ?? null;

    const portLimits = ports?.portsTaken.portLimits ?? [1024,65535];

    const [alter,{ error: errorM }] = useMutation<{alterServer: boolean}>(gql`
        mutation AlterServer($name: String!, $maxMemory: Float, $upCmd: String, $port: Int, $password: String!) {
            alterServer(name: $name, maxMemory: $maxMemory, upCmd: $upCmd, port: $port, password: $password)
        }
    `);

    const mutate = async (
        rest: {
            maxMemory: number | null,
            port: number | null
        },
        password: string
    ) => {
        return await alter({
            variables: {
                password,
                name: pageData.name,
                ...rest
            }
        }); 
    }
    
    const schema = useMemo(() => {
        return {
            type: "object",
            properties: {
                maxMemory: NumberInputData(1,32),
                upCmd: { type: ["string", "null"] },
                port: NumberInputData(portLimits[0],portLimits[1])
            },
            additionalProperties: false
        }
    },[portLimits]);

    const { 
        register, 
        handleSubmit,
        control,
        formState: { errors }
    } = useForm<FormData>({
        resolver: ajvResolver(schema as any, {
            formats: fullFormats
        })
    });

    const onSubmit = async (fd: FormData) => {
        
        let password = prompt("Please enter the password to alter this server");
        
        if (!password) {
            return;
        }

        let data = {
            maxMemory: fd.maxMemory.value,
            port: fd.port.value
        };
        
        let res = await mutate(data, password);

        if (res.data?.alterServer) {
            window.location.href = `/`;
        }
    };

    if (loading) return <Spinner />;
    if (error) return <pre>Error: {error.message}</pre>;

    return (
        <form onSubmit={handleSubmit(onSubmit)}>
            <p><HomeLink href="/">Home</HomeLink><TextBig>Alter {instanceData?.name} page: </TextBig><Btn type="submit" >Change server</Btn></p>
            {errorM && <ErrorP>{errorM.message}</ErrorP>}

            <Label>Max Memory, (1;32)</Label><br />
            <NumberInput type="float" name="maxMemory" control={control} placeholder={instanceData?.max_memory?.toString() ?? "-"} /><br />
            {errors.maxMemory && <ErrorP>{errors.maxMemory.message}</ErrorP>}

            
            <Label>Port, <DisplayRange range={portLimits}/></Label><br />
            <NumberInput type="int" name="port" control={control} placeholder={instanceData?.port?.toString() ?? "-"} /><br />
            {errors.port && <ErrorP>{errors.port.message}</ErrorP>}
        </form>
    );
}


window.onload = makeOnLoad(Alter);