import fs from "fs";
import path from "path";

type LogLevel = "INFO" | "WARN" | "ERROR" | "DEBUG";

let hasInitialized = false;

function formatMessage(level: LogLevel, args: unknown[]): string {
  const timestamp = new Date().toISOString();
  const content = args
    .map((arg) => {
      if (arg instanceof Error) {
        return `${arg.name}: ${arg.message}\n${arg.stack || ""}`.trim();
      }
      if (typeof arg === "string") {
        return arg;
      }
      try {
        return JSON.stringify(arg);
      } catch {
        return String(arg);
      }
    })
    .join(" ");

  return `[${timestamp}] [${level}] ${content}`;
}

export function initScriptLogger(platform: string, outputDir: string): string {
  const logDir = path.join(outputDir, "logs");
  fs.mkdirSync(logDir, { recursive: true });

  const logFile = path.join(logDir, `login-script-${platform}.log`);

  if (hasInitialized) {
    return logFile;
  }

  hasInitialized = true;

  const writeLine = (level: LogLevel, args: unknown[]) => {
    const line = formatMessage(level, args);
    fs.appendFileSync(logFile, `${line}\n`, { encoding: "utf-8" });
  };

  const originalLog = console.log.bind(console);
  const originalWarn = console.warn.bind(console);
  const originalError = console.error.bind(console);
  const originalDebug = console.debug.bind(console);

  console.log = (...args: unknown[]) => {
    writeLine("INFO", args);
    originalLog(...args);
  };

  console.warn = (...args: unknown[]) => {
    writeLine("WARN", args);
    originalWarn(...args);
  };

  console.error = (...args: unknown[]) => {
    writeLine("ERROR", args);
    originalError(...args);
  };

  console.debug = (...args: unknown[]) => {
    writeLine("DEBUG", args);
    originalDebug(...args);
  };

  process.on("unhandledRejection", (reason) => {
    writeLine("ERROR", ["未处理的 Promise 拒绝", reason]);
  });

  process.on("uncaughtException", (error) => {
    writeLine("ERROR", ["未捕获异常", error]);
  });

  writeLine("INFO", [`登录脚本日志初始化完成: ${logFile}`]);
  return logFile;
}
