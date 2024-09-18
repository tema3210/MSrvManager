export enum ServerState {
    "RUNNING",
    "STOPPED",
    "CRASHED"
}

export type InstanceDescriptor = {
    name: string,
    mods: string,
    state: ServerState,

    memory: number,
    maxMemory: number,
    port: number
}