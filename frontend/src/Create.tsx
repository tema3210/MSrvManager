import { useMutation } from "@apollo/client";
import gql from "graphql-tag";
import { ChangeEventHandler, useState } from "react";
import { makeOnLoad } from "./lib";
import { useForm } from "react-hook-form";

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

const CreatePage = () => {

    const form = useForm<NewServerReq>();

    const [createServer, {data,loading,error}] = useMutation<NewServerReq>(gql`
        mutation Mutation($data: NewServer!) {
            newServer(data: $data)
        }
    `);

    const onSubmit = (formData: NewServerReq) => {
        createServer({ variables: { data: formData } });
    };

    return <>
        Create server page
        <form onSubmit={form.handleSubmit(onSubmit)}>
            <input type="text" {...form.register("name")} placeholder="server name" /><br />
            <input type="text" {...form.register("upCmd")} placeholder="command by which it can be launched" /><br />
            <input type="text" {...form.register("setupCmd")} placeholder="command run once at the root of archive" /><br />
            <input type="text" {...form.register("url")} placeholder="url to mod list" /><br />
            <input type="number" {...form.register("maxMemory")} placeholder="maximum allowed memory consumption" /><br />
            <input type="number" {...form.register("port")} placeholder="server port" /><br />
            <input type="number" {...form.register("rcon")} placeholder="server rcon" /><br />
            <label>Archive with server instance (max 500 MB)</label>
            <input type="file" {...form.register("instanceUpload")} /><br /> 
            <button type="submit" disabled={loading} >test create server (with file)</button>
        </form>
        {data && <p>Server created successfully!</p>}
        {error && <p>Error creating server: {error.message}</p>}
        
    </>
}

window.onload = makeOnLoad(<CreatePage />)