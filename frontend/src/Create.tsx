import { useMutation } from "@apollo/client";
import gql from "graphql-tag";
import { ChangeEventHandler, useState } from "react";
import { makeOnLoad } from "./lib";
import { InstanceDescriptor } from "./model";

const CreatePage = () => {

    const [vars,setVars] = useState<object | null>();

    const [createServer] = useMutation(gql`
        mutation Mutation($file: Upload!) {
            newServer(data: {
                name: "test big",
                upCmd: "echo hi",
                url: "http://google.com",
                maxMemory: 1.5,
                port: 25566,
                rcon: 26002,
                instanceUpload: $file
            })
        }
        `,
        {
            variables: vars ?? {}
        }
    );

    const onChange: ChangeEventHandler<HTMLInputElement> = (ev) => {
        let file = ev.target.files?.[0];

        if (file) {
            setVars({
                file
            })
        } else {
            setVars(null)
        }
    };

    return <>
        Create server page
        <input type="file" onChange={onChange} /><br /> 
        <button onClick={() => createServer()}>test create server (with file)</button>
    </>
}

window.onload = makeOnLoad(<CreatePage />)