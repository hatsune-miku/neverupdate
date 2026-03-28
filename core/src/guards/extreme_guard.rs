use crate::error::Result;
use crate::extreme;
use crate::guards::GuardPoint;
use crate::model::GuardPointStatus;
use crate::pathing::software_distribution_path;
use crate::state::PersistedState;
use crate::ti_service::TiService;

pub struct ExtremeGuard;

impl GuardPoint for ExtremeGuard {
    fn id(&self) -> &'static str {
        "extreme_mode"
    }

    fn title(&self) -> &'static str {
        "极端手段"
    }

    fn description(&self) -> &'static str {
        "彻底破坏 Windows Update 数据目录（不参与一键操作，执行要求二次确认）"
    }

    fn batch_eligible(&self) -> bool {
        false
    }

    fn tracks_desired(&self) -> bool {
        false
    }

    fn check(&self, _state: &PersistedState) -> Result<GuardPointStatus> {
        let target = software_distribution_path();
        let executed = target.exists() && target.is_file();

        let message = if executed {
            Some(String::from("极端手段已执行：SoftwareDistribution 已被替换为文件"))
        } else {
            Some(String::from("未执行。该阻断点不参与任何一键操作，且执行时要求二次确认"))
        };

        Ok(self.build_status(executed, message))
    }

    fn guard(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        extreme::run_extreme_mode(state, ti)?;
        ti.as_admin(|| self.check(state))
    }

    fn release(&self, _state: &mut PersistedState, _ti: &TiService) -> Result<GuardPointStatus> {
        _ti.as_admin(|| self.check(_state))
    }
}
