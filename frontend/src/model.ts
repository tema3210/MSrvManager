export type ServerState = "Running" | "Stopped" | "Crashed"

export type InstanceDescriptor = {
    name: string,
    mods: string,
    state: ServerState,

    memory: number | null,
    
    java_args: string[],
    max_memory: number,
    port: number
}

export type PortTaken = {
    ports: number[],
    rcons: number[]
}

export type PortLimits = {
    portLimits: [number,number],
    rconLimits: [number,number]
}