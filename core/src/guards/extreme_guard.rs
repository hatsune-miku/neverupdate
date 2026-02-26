use crate::error::Result;
use crate::extreme;
use crate::guards::status_extreme;
use crate::model::GuardPointStatus;
use crate::pathing::software_distribution_path;

pub fn check() -> Result<GuardPointStatus> {
    let target = software_distribution_path();
    let executed = target.exists() && target.is_file();

    let message = if executed {
        Some(String::from(
            "极端手段已执行：SoftwareDistribution 已被替换为文件",
        ))
    } else {
        Some(String::from(
            "未执行。该阻断点不参与任何一键操作，且执行时要求二次确认",
        ))
    };

    Ok(status_extreme(executed, message))
}

pub fn guard() -> Result<GuardPointStatus> {
    extreme::run_extreme_mode()?;
    check()
}

pub fn release() -> Result<GuardPointStatus> {
    check()
}
