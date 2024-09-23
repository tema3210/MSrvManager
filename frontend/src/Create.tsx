import { useMutation, useQuery } from "@apollo/client";
import gql from "graphql-tag";
import { ajvResolver } from '@hookform/resolvers/ajv';
import { makeOnLoad, SSRProps } from "./lib";
import { useForm } from "react-hook-form";
import { ErrorP, Label, NumberInput, SInput, TextBig } from "./components/UIComps";
import { PortsInfo } from "./model";
import { useMemo } from "react";
import { fullFormats } from "ajv-formats/dist/formats";
import Btn from "./components/Button";

type NewServerReq = {
    name: string,
    upCmd: string,
    setupCmd: string | null,
    url: string,
    maxMemory: number,
    port: number,
    rcon: number,
    instanceUpload: FileList
}

type FormData = Record<keyof NewServerReq, string | number | object | null>;

const DisplayRange = ({range}:{range: [number,number]}) => (<>, allowed: ({range[0]};{range[1]})</>)

const CreatePage = ({}: SSRProps) => {

    const { data: ports, loading: pLoading } = useQuery<PortsInfo>(gql`
        {
            portsTaken {
                ports,
                rcons,
                portLimits,
                rconLimits
            }
        }
    `);

    const portLimits = [ports?.portsTaken.portLimits[0] ?? 1, ports?.portsTaken.portLimits[1] ?? 65535];
    
    const rconLimits = [ports?.portsTaken.rconLimits[0] ?? 1, ports?.portsTaken.rconLimits[1] ?? 65535];

    const NumberInputData = (minimum: number,maximum: number) => ({
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

    //fix the schema
    const schema = useMemo(() => {
        return {
            type: "object",
            properties: {
              name: { "type": "string" },
              upCmd: { "type": "string" },
              setupCmd: { 
                type: ["string", "null"]
              },
              url: { type: "string", format: "uri" },
              maxMemory: NumberInputData(1.0, 32.0),
              port: NumberInputData(ports?.portsTaken.portLimits[0] ?? 1, ports?.portsTaken.portLimits[1] ?? 65535),
              rcon: NumberInputData(ports?.portsTaken.rconLimits[0] ?? 1, ports?.portsTaken.rconLimits[1] ?? 65535),
              // instanceUpload: { 
              //   type: "array",
              //   items: {
              //     type: "object"
              //   },
              //   maxItems: 1,
              //   minItems: 1,
              // },

            },
            required: ["name", "upCmd", "url", "maxMemory", "port", "rcon", "instanceUpload"]
          }
    },[ports]);

    const {
        register,
        control,
        handleSubmit,
        formState: { errors }
      } = useForm<FormData>({
        resolver: ajvResolver(schema as any, {
            formats: fullFormats
        }),
        defaultValues: {
          name: "",
          upCmd: "",
          setupCmd: null,
          url: "",
          instanceUpload: null
        }
      });

    const [createServer, {data,loading: csLoading,error}] = useMutation<any>(gql`
        mutation Mutation($data: NewServer!) {
            newServer(data: $data)
        }
    `);

    const onSubmit = async (formData: FormData) => {
        let data = {
          ...formData,
          instanceUpload: (formData.instanceUpload as unknown as FileList | undefined)?.[0] ?? null,
          maxMemory: (formData.maxMemory as any)?.value ?? null,
          port: (formData.port as any)?.value ?? null,
          rcon: (formData.rcon as any)?.value ?? null
        };

        const result = await createServer({
          variables: {
            data
          }
        });

        if (result.data?.newServer) {
          // if all is fine then go back to index
          window.location.href = '/';
        }
    
    };

    return <>
        
        <form onSubmit={handleSubmit(onSubmit,(e) => {console.log("Ehm?",e)})}>
            <p><TextBig>Create server page: </TextBig><Btn type="submit" disabled={csLoading || pLoading} >Create server</Btn></p>

            <Label>Name</Label><br />
            <SInput type="text" {...register("name")} placeholder="server name" /><br />
            {errors.name && <ErrorP>{errors.name.message}</ErrorP>}

            <Label>Command to launch the instance</Label><br />
            <SInput type="text" {...register("upCmd")} placeholder="command by which it can be launched" /><br />
            {errors.upCmd && <ErrorP>{errors.upCmd.message}</ErrorP>}

            <Label>Setup command to be run once</Label><br />
            <SInput type="text" {...register("setupCmd")} placeholder="command run once at the root of archive" /><br />
            {errors.setupCmd && <ErrorP>{errors.setupCmd.message}</ErrorP>}

            <Label>Url to client modpack</Label><br />
            <SInput type="text" {...register("url")} placeholder="url to mod list" /><br />
            {errors.url && <ErrorP>{errors.url.message}</ErrorP>}

            <Label>Maximum memory, in GB (1..32)</Label><br />
            <NumberInput name="maxMemory" type="float" control={control} placeholder="max allowed memory consumption" /><br />

            <Label>Port{ports?.portsTaken.portLimits ? <DisplayRange range={ports.portsTaken.portLimits}/> : null}</Label><br />
            <NumberInput name="port" type="int" control={control} placeholder="server port" /><br />

            <Label>Rcon{ports?.portsTaken.rconLimits ? <DisplayRange range={ports.portsTaken.rconLimits}/> : null}</Label><br />
            <NumberInput name="rcon" type="int" control={control} placeholder="server rcon" /><br />

            <Label>Archive with server instance (max 500 MB)</Label><br /> 
            <SInput type="file" {...register("instanceUpload")} /><br /> 
            {error && <ErrorP>{error.message}</ErrorP>}
        </form>    
    </>
}

window.onload = makeOnLoad(CreatePage)