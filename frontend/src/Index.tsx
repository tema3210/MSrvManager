import { useEffect, useState } from "react";
import { makeOnLoad } from "./lib";

import { gql, useMutation, useSubscription, useQuery } from "@apollo/client";

const Index = ({}) => {

    // const {data: apiV, loading: lV} = useQuery(gql`
    //     apiVersion
    // `);

    const { data, loading, error } = useSubscription(gql`
        subscription {
            servers {
                name,
                memory,
                maxMemory
            }
        }
    `);

    const [file,setFile] = useState<any>(null);

    useEffect(() => console.log("file:",file),[file]);

    const [createServer] = useMutation(gql`
        mutation Mutation($file: Upload!) {
            newServer(
                name: "test",
                cmds: {
                    up: "echo hi",
                    setup: "echo bye"
                },
                url: "http://google.com",
                maxMemory: 1.5,
                port: 25565,
                rcon: 26001,
                instanceUpload: $file
            )
        }
        `,
        {
            variables: {
                file
            }
        }
    );

    if (loading) return "Loading server list";
    if (error) return <pre>{error.message}</pre>

    return <>
        <p>Hello, we have {JSON.stringify(data)}</p>
        {/* <p>{(lV)? JSON.stringify(apiV) : null}</p> */}
        <input type="file" onChange={(ev) => setFile(ev.target.value)} /><br />
        <button onClick={() => createServer()}>test create server (with file)</button>
    </>
}

window.onload = makeOnLoad(<Index />)
