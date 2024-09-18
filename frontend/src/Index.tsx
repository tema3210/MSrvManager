import { ChangeEventHandler, useEffect, useState } from "react";
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

    const [vars,setVars] = useState<object | null>();

    const [createServer] = useMutation(gql`
        mutation Mutation($file: Upload!) {
            newServer(data: {
                name: "test",
                upCmd: "echo hi",
                url: "http://google.com",
                maxMemory: 1.5,
                port: 25565,
                rcon: 26001,
                instanceUpload: $file
            })
        }
        `,
        {
            variables: vars ?? {}
        }
    );

    if (loading) return "Loading server list";
    if (error) return <pre>{error.message}</pre>

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
        <p>Hello, we have {JSON.stringify(data)}</p>
        {/* <p>{(lV)? JSON.stringify(apiV) : null}</p> */}
        <input type="file" onChange={onChange} /><br />
        <button onClick={() => createServer()}>test create server (with file)</button>
    </>
}

window.onload = makeOnLoad(<Index />)
