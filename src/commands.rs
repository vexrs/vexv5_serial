
pub enum ExtCommand {
    FileCtrl = 0x10,
    FileInit = 0x11,
    FileExit = 0x12,
    FileWrite = 0x13,
    FileRead = 0x14,
    FileLink = 0x15,
    FileDir = 0x16,
    FileDirEntry = 0x17,
    FileLoad = 0x18,
    FileGetInfo = 0x19,
    FileSetInfo = 0x1A,
    FileErase = 0x1b,
    FileStat = 0x1c,
    FileVisualize = 0x1d,
    FileCleanup = 0x1e,
    FileFormat = 0x1f,
    SystemFlags = 0x20,
    DeviceStatus = 0x21,
    SystemStatus = 0x22,
    FDTStatus = 0x23,
    LogStatus = 0x24,
    LogRead = 0x25,
    RadioStatus = 0x26,
    UserRead = 0x27,
    ScreenCapture = 0x28,
    UserProgram = 0x29,
    DashTouch = 0x2a,
    DashSelect = 0x2b,
    DashEnable = 0x2c,
    DashDisable = 0x2d,
    SystemKeyValueWrite = 0x2e,
    SystemKeyValueSave = 0x2f,
    Slots1to4Info = 0x31,
    Slots5to8Info  = 0x32,

    // DANGER: Use factory commands at own risk
    FactoryStatus = 0xf1,
    FactoryReset = 0xf2,
    FactoryPing = 0xf3,
    FactoryPong = 0xf4,
    FactoryChallenge = 0xfc,
    FactoryResponse = 0xfd,
    FactorySpecial = 0xfe,
    FactoryEnable = 0xff,
}

pub enum SimpleCommand {
    Extended = 0x56
}

struct Command<T> {
    command: SimpleCommand,
    ext_command: Option<ExtCommand>,
    fields: T
}