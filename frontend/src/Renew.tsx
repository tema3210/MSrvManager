import { useMutation, useQuery } from "@apollo/client";
import gql from "graphql-tag";
import { ajvResolver } from '@hookform/resolvers/ajv';
import { makeOnLoad, SSRProps } from "./lib";
import { useForm } from "react-hook-form";
import { DisplayRange, ErrorP, HomeLink, Label, NumberInput, SInput, TextArea, TextBig } from "./components/UIComps";
import { useMemo, useState } from "react";
import { fullFormats } from "ajv-formats/dist/formats";
import Btn from "./components/Button";
import { NumberInputData } from "./schema_utils";
import { PortLimits, PortTaken } from "./model";

type ServerData = {
    // setupCmd: string | null,
    // serverJar: string,
    javaArgs: string | null,

    url: string,
    maxMemory: number,

    ports: {
        port: number,
        rcon: number
    },
}

type SpecialHandling = "maxMemory";

type FormData = Omit<ServerData, SpecialHandling> & {
    name: string,
    maxMemory: {value: number, displayValue: string},
    port: {value: number, displayValue: string},
    rcon: {value: number, displayValue: string},
    instanceUpload: {value: FileList, formData: File[]} | null
};

type PageProps = {
  name: string,
  data: any
}

const ReNewPage = ({ pageData }: SSRProps<PageProps>) => {

    console.log("PageData:",pageData);

    const { data: ports, loading: pLoading } = useQuery<{portsTaken: PortLimits}>(gql`
        {
            portsTaken {
                portLimits,
                rconLimits
            }
        }
    `);

    const memoryLimits: [number,number] = [1,8];

    const schema = useMemo(() => {
        return {
            type: "object",
            properties: {
              // serverJar: { "type": "string", minLength: 7 }, // aka ./_.jar
              upCmd: { "type": "string" },
              javaArgs: { "type": "string", minLength: 10 },
              setupCmd: {
                oneOf: [
                  { type: "string" },
                  { type: "null" }
                ]
              },
              url: { type: "string", format: "uri" },
              maxMemory: NumberInputData(memoryLimits[0],memoryLimits[1]),
              port: NumberInputData(ports?.portsTaken.portLimits[0] ?? 1, ports?.portsTaken.portLimits[1] ?? 65535),
              rcon: NumberInputData(ports?.portsTaken.rconLimits[0] ?? 1, ports?.portsTaken.rconLimits[1] ?? 65535),
            },
            required: ["javaArgs", "url", "maxMemory", "port", "rcon"]
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
        // serverJar: "",
        // setupCmd: null,
        url: "",
        instanceUpload: null
      }
    });

    const [renewServer, {error}] = useMutation<any>(gql`
        mutation Mutation($name: String!, $data: NewServer!,$upload: Upload!, $password: String!) {
            reNewServer(name: $name,data: $data, password: $password)
        }
    `);

    const onSubmit = async (formData: FormData) => {
        let name = pageData.name;

        let data: ServerData = {
          javaArgs: formData.javaArgs?.length === 0 ? null : formData.javaArgs,
          url: formData.url,
          // setupCmd: formData.setupCmd?.length === 0 ? null : formData.setupCmd,
          maxMemory: (formData.maxMemory as any)?.value ?? null,
          ports: {
            port: (formData.port as any)?.value ?? null,
            rcon: (formData.rcon as any)?.value ?? null
          }
        };

        let password = prompt("Please enter the password to renew the server");

        if (!password) {
          return;
        }

        let upload = formData.instanceUpload?.formData[0]!;

        const result = await renewServer({
          variables: {
            name,
            data,
            upload,
            password
          }
        });


        if (result.data?.reNewServer) {
          // if all is fine then go back to index
          window.location.href = '/';
        }
    
    };
    
    return <>
        <form onSubmit={handleSubmit(onSubmit,(e) => console.log("Es:",e))}> 
            <p><HomeLink href="/">Home</HomeLink><TextBig>Renew server page: </TextBig><Btn type="submit" disabled={pLoading} >Renew server</Btn></p>
            {error && <ErrorP>{error.message}</ErrorP>}

            {/* <Label>Path to jar in archive to be executed as a server</Label><br />
            <SInput type="text" {...register("serverJar")} placeholder="relative path required, aka ./_.jar" /><br />
            {errors.serverJar && <ErrorP>{errors.serverJar.message}</ErrorP>} */}

            <Label>Paramaters for JVM, -Xmx, -Xms, classpath excluded</Label><br />
            <TextArea name="javaArgs" control={control} placeholder="JVM params" /><br />
            {errors.javaArgs && <ErrorP>{errors.javaArgs.message}</ErrorP>}


            <Label>Url to client modpack</Label><br />
            <SInput type="text" {...register("url")} placeholder="url to mod list" /><br />
            {errors.url && <ErrorP>{errors.url.message}</ErrorP>}

            <Label>Maximum memory, in GB <DisplayRange range={memoryLimits}/> </Label><br />
            <NumberInput name="maxMemory" type="float" control={control} placeholder="max allowed memory consumption" /><br />

            <Label>Port, {ports?.portsTaken.portLimits ? <DisplayRange range={ports.portsTaken.portLimits}/> : null}</Label><br />
            <NumberInput name="port" type="int" control={control} placeholder="server port" /><br />

            <Label>Rcon, {ports?.portsTaken.rconLimits ? <DisplayRange range={ports.portsTaken.rconLimits}/> : null}</Label><br />
            <NumberInput name="rcon" type="int" control={control} placeholder="server rcon" /><br />

        </form>    
    </>
}

window.onload = makeOnLoad(ReNewPage)