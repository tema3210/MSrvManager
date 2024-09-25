import Spinner from "./components/Spinner";
import { makeOnLoad, SSRProps } from "./lib";
import { useQuery, gql, useMutation } from '@apollo/client';
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
    upCmd: string | null;
    port: {value: number, displayValue: string};
};

const Alter = ({pageData}: SSRProps) => {

    const { loading, error, data } = useQuery<{instance: InstanceDescriptor | null}>(
        gql`
            query Instance($name: String!) {
                instance(name: $name) {
                    name
                    maxMemory
                    port
                }
            }
        `, 
        {
            variables: {
                name: pageData.name
            }
        }
    );

    const { data: ports } = useQuery<{ portsTaken: {portLimits: [number,number]} }>(gql`
        {
            portsTaken {
                portLimits,
            }
        }
    `);

    const instanceData = data?.instance ?? null;

    const portLimits = ports?.portsTaken.portLimits ?? [1024,65535];

    const [alter,{ error: errorM }] = useMutation<{alterServer: boolean}>(gql`
        mutation AlterServer($name: String!, $maxMemory: Float, $upCmd: String, $port: Int) {
            alterServer(name: $name, maxMemory: $maxMemory, upCmd: $upCmd, port: $port)
        }
    `);

    const mutate = async (
        rest: {
            maxMemory: number | null,
            upCmd: string | null, 
            port: number | null
        }
    ) => {
        return await alter({
            variables: {
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
        }),
        defaultValues: {
            upCmd: null
        }
    });

    const onSubmit = async (fd: FormData) => {
        
        let data = {
            maxMemory: fd.maxMemory.value,
            upCmd: (fd.upCmd === "")? null : fd.upCmd,
            port: fd.port.value
        };
        
        let res = await mutate(data);

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

            <Label>Max Memory</Label><br />
            <NumberInput type="float" name="maxMemory" control={control} placeholder={instanceData?.maxMemory?.toString() ?? "-"} /><br />
            {errors.maxMemory && <ErrorP>{errors.maxMemory.message}</ErrorP>}

            <Label htmlFor="upCmd">Up Command</Label><br />
            <SInput id="upCmd" type="text" {...register("upCmd")} /><br />
            {errors.upCmd && <ErrorP>{errors.upCmd.message}</ErrorP>}
            
            <Label>Port, <DisplayRange range={portLimits}/></Label><br />
            <NumberInput type="int" name="port" control={control} placeholder={instanceData?.port?.toString() ?? "-"} /><br />
            {errors.port && <ErrorP>{errors.port.message}</ErrorP>}
        </form>
    );
}


window.onload = makeOnLoad(Alter);