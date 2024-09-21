import { useMutation, useQuery } from "@apollo/client";
import gql from "graphql-tag";
import { ajvResolver } from '@hookform/resolvers/ajv';
import { makeOnLoad, SSRProps } from "./lib";
import { useForm } from "react-hook-form";
import { ErrorP, NumberInput, SInput, TextBig } from "./components/UIComps";
import { PortsInfo } from "./model";
import { useMemo } from "react";
import { fullFormats } from "ajv-formats/dist/formats";

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

type FormData = Record<keyof NewServerReq, string | number | null>;

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
              // it should be a float, but schema validation doesn't support it
              maxMemory: {
                type: "number",
                // format: "float",
                // minimum: 1.0 
              },
              port: {
                type: "number", 
                minimum: ports?.portsTaken.portLimits[0] ?? 1, 
                maximum: ports?.portsTaken.portLimits[1] ?? 65535 
              },
              rcon: { 
                type: "number", 
                minimum: ports?.portsTaken.rconLimits[0] ?? 1, 
                maximum: ports?.portsTaken.rconLimits[1] ?? 65535  
              },
              instanceUpload: { 
                type: "array",
                items: {
                  type: "object"
                },
                maxItems: 1,
                minItems: 1,
              },

            },
            required: ["name", "upCmd", "url", "maxMemory", "port", "rcon"]
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

    const [createServer, {data,loading: csLoading,error}] = useMutation<boolean>(gql`
        mutation Mutation($data: NewServer!) {
            newServer(data: $data)
        }
    `);

    const onSubmit = (data: any) => {

        let instanceUpload = data.instanceUpload?.[0] ?? null;

        createServer({ 
          variables: { 
            data: {
              ...data,
              instanceUpload
            } 
          } 
        });
    };

    return <>
        <TextBig>Create server page:</TextBig>
        <form onSubmit={handleSubmit(onSubmit)}>
            <label>Name</label><br />
            <SInput type="text" {...register("name")} placeholder="server name" /><br />
            {errors.name && <ErrorP>{errors.name.message}</ErrorP>}

            <label>Command to launch the instance</label><br />
            <SInput type="text" {...register("upCmd")} placeholder="command by which it can be launched" /><br />
            {errors.upCmd && <ErrorP>{errors.upCmd.message}</ErrorP>}

            <label>Setup command to be run once</label><br />
            <SInput type="text" {...register("setupCmd")} placeholder="command run once at the root of archive" /><br />
            {errors.setupCmd && <ErrorP>{errors.setupCmd.message}</ErrorP>}

            <label>Url to client modpack</label><br />
            <SInput type="text" {...register("url")} placeholder="url to mod list" /><br />
            {errors.url && <ErrorP>{errors.url.message}</ErrorP>}

            <label>Maximum memory, in GB</label><br />
            <NumberInput name="maxMemory" type="float" control={control} placeholder="max allowed memory consumption" /><br />

            <label>Port{ports?.portsTaken.portLimits ? <DisplayRange range={ports.portsTaken.portLimits}/> : null}</label><br />
            <NumberInput name="port" type="int" control={control} placeholder="server port" /><br />

            <label>Rcon{ports?.portsTaken.rconLimits ? <DisplayRange range={ports.portsTaken.rconLimits}/> : null}</label><br />
            <NumberInput name="rcon" type="int" control={control} placeholder="server rcon" /><br />

            <label>Archive with server instance (max 500 MB)</label><br /> 
            <SInput type="file" {...register("instanceUpload")} /><br /> 
            <button type="submit" disabled={csLoading || pLoading} >Create server</button>
        </form>
        {data && <p>Server created successfully!</p>}
        {error && <p>Error creating server: {error.message}</p>}
        
    </>
}

window.onload = makeOnLoad(CreatePage)