import { useState } from "react";
import { makeOnLoad, SSRProps } from "./lib";
import { InstanceDescriptor } from "./model";
import InstanceDisplay from "./components/InstanceDesc";
import InstanceActions from "./components/InstanceActions";
import Btn from "./components/Button";

import { gql, useSubscription, useQuery } from "@apollo/client";
import styled from "styled-components";

const Wrapper = styled.div`
    display: flex;
    flex-direction: row;
    align-items: stretch;
`;

const InstanceWrapper = styled.div`
    margin-left: 1rem;
    margin-right: 1rem;
    width: calc(45% - 2rem);
`;

const TextBig = styled.div`
    font-size: 2rem;
    color: #db9f30;
    /* margin-left: 2rem; */
    /* width: calc(100% - 2rem); */
`;

const Footer = styled.div`
    position: absolute;
    bottom: 0;
    height: 3rem;
`;

const Index = ({}: SSRProps) => {

    const { data: AVdata, loading: AVloading } = useQuery(gql`
        query {
            appVersion
        }
    `);

    const { data, loading, error } = useSubscription<{servers: InstanceDescriptor[]}>(gql`
        subscription {
            servers {
                name,
                memory,
                state,
                maxMemory,
                mods,
                port
            }
        }
    `);

    const [selected,setSelected] = useState<InstanceDescriptor | null>(null);

    if (loading) return "Loading server list";
    if (error) return <pre>{error.message}</pre>

    const createOnClick = () => {
        window.location.href = '/create';
    };
    
    return <Wrapper>
        <InstanceWrapper>
            <TextBig>We have these servers:</TextBig>
            {(data?.servers ?? [])
                .map((v) => (
                    <InstanceDisplay
                        key={v.name}
                        instance={v}
                        selected={selected?.name === v.name}
                        setSelected={
                            (selected?.name === v.name)
                                ? () => setSelected(null)
                                : () => setSelected(v)
                        }
                    />
                ))
            }
        </InstanceWrapper>
        <InstanceWrapper>
            <TextBig>Actions:</TextBig>
            <Btn onClick={createOnClick}>Create Server =&gt;</Btn>
            {
                (selected) ? <InstanceActions instance={selected}/> : null
            }
        </InstanceWrapper>
        
        <Footer><TextBig>Version: {(AVloading)? "-" : AVdata.appVersion }</TextBig></Footer>
    </Wrapper>
}

window.onload = makeOnLoad(Index)
