use asher_ipc::{IpcRequest, IpcResponse, send_request};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum OutputCommand {
    List,
    Scale {
        scale: f64,
        #[arg(long)]
        output: Option<String>,
    },
}

pub fn list_outputs(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    match send_request(&IpcRequest::ListOutputs)? {
        IpcResponse::Outputs { outputs } => {
            if json {
                println!("{}", serde_json::to_string_pretty(&outputs)?);
            } else {
                for output in outputs {
                    let primary = if output.primary { "*" } else { " " };
                    let refresh = output.refresh_millihertz as f64 / 1000.0;
                    println!(
                        "{}{}\t{}x{}\t{:.2}x\t{refresh:.3}Hz\t{} {}",
                        primary,
                        output.name,
                        output.width,
                        output.height,
                        output.scale,
                        output.make,
                        output.model
                    );
                }
            }
            Ok(())
        }
        IpcResponse::Error { message } => Err(message.into()),
        response => Err(format!("unexpected response: {response:?}").into()),
    }
}

pub fn set_output_scale(
    output: Option<String>,
    scale: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    match send_request(&IpcRequest::SetOutputScale { output, scale })? {
        IpcResponse::Accepted => Ok(()),
        IpcResponse::Error { message } => Err(message.into()),
        response => Err(format!("unexpected response: {response:?}").into()),
    }
}
