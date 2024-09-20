import { useMutation, useQuery } from "@apollo/client";
import gql from "graphql-tag";
import { ChangeEventHandler, useState } from "react";
import { makeOnLoad, SSRProps } from "./lib";
import { useForm } from "react-hook-form";
import styled from "styled-components";
import { TextBig } from "./components/UIComps";
import { PortsInfo } from "./model";

type NewServerReq = {
    name: string,
    upCmd: string,
    setupCmd: string | null,
    url: string,
    maxMemory: number,
    port: number,
    rcon: number,
    instanceUpload: File
}

type FormData = Record<keyof NewServerReq,any>;

const SInput = styled.input`
    margin-bottom: 0.5rem;
    width: 17rem;
`;

const CreatePage = ({}: SSRProps) => {

    const form = useForm<FormData>();

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

    console.log("got ports:",ports, pLoading);

    const [createServer, {data,loading: csLoading,error}] = useMutation<boolean>(gql`
        mutation Mutation($data: NewServer!) {
            newServer(data: $data)
        }
    `);

    const onSubmit = (formData: FormData) => {

        console.log("formData:",formData);

        let data: NewServerReq = {
            ...formData,
            maxMemory: parseFloat(formData.maxMemory),
            port: parseInt(formData.maxMemory),
            rcon: parseInt(formData.maxMemory),
            instanceUpload: formData.instanceUpload[0] ?? null
        }

        createServer({ variables: { data } });
    };

    return <>
        <TextBig>Create server page:</TextBig>
        <form onSubmit={form.handleSubmit(onSubmit)}>
            <SInput type="text" {...form.register("name")} placeholder="server name" /><br />
            <SInput type="text" {...form.register("upCmd")} placeholder="command by which it can be launched" /><br />
            <SInput type="text" {...form.register("setupCmd")} placeholder="command run once at the root of archive" /><br />
            <SInput type="text" {...form.register("url")} placeholder="url to mod list" /><br />
            <SInput type="number" step="0.2" {...form.register("maxMemory")} placeholder="max allowed memory consumption" /><br />
            <SInput type="number" {...form.register("port")} placeholder="server port" /><br />
            <SInput type="number" {...form.register("rcon")} placeholder="server rcon" /><br />
            <label>Archive with server instance (max 500 MB)</label><br /> 
            <SInput type="file" {...form.register("instanceUpload")} /><br /> 
            <button type="submit" disabled={csLoading || pLoading} >Create server</button>
        </form>
        {data && <p>Server created successfully!</p>}
        {error && <p>Error creating server: {error.message}</p>}
        
    </>
}

window.onload = makeOnLoad(CreatePage)