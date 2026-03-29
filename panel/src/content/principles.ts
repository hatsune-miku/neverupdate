export interface GuardPrinciple {
  title: string
  markdown: string
}

const PRINCIPLES: Record<string, GuardPrinciple> = {
  service_watchdog: {
    title: '干扰更新服务',
    markdown: [
      '### 这个阻断点会做什么',
      '- 将 `WaaSMedicSvc`、`UsoSvc`、`uhssvc` 的启动类型改为禁用。',
      '- 同时把服务 `ImagePath` 前缀改成 `DISABLE:`，让系统即使尝试拉起也难以成功。',
      '- 释放时会根据备份恢复原始配置。',
      '',
      '### 需要动用的权限级别',
      '- 需要 **TrustedInstaller + 管理员** 权限。',
      '- 原因：这些服务位于系统核心服务树，普通管理员写入常被拒绝或被快速回滚。',
    ].join('\n'),
  },
  hosts_firewall: {
    title: 'Hosts 与防火墙',
    markdown: [
      '### 这个阻断点会做什么',
      '- 在 `hosts` 中写入更新域名映射到 `127.0.0.1`。',
      '- 新增防火墙出站阻断规则，拦截相关更新进程联网。',
      '- 形成“名称解析 + 出站访问”双层阻断。',
      '',
      '### 需要动用的权限级别',
      '- 需要 **TrustedInstaller + 管理员** 权限。',
      '- 原因：`hosts` 和防火墙策略都属于系统级资源，写入与校验都需要高权限上下文。',
    ].join('\n'),
  },
  group_policy: {
    title: '组策略',
    markdown: [
      '### 这个阻断点会做什么',
      '- 写入 Windows Update 相关组策略项，关闭自动更新路径。',
      '- 同步设置策略管理器中的设备级更新访问限制。',
      '- 让系统侧更新入口在策略层面保持“默认关闭”。',
      '',
      '### 需要动用的权限级别',
      '- 需要 **TrustedInstaller + 管理员** 权限。',
      '- 原因：策略键在 HKLM 下，且会被系统策略组件持续读取并可能纠偏。',
    ].join('\n'),
  },
  scheduled_tasks: {
    title: '计划任务',
    markdown: [
      '### 这个阻断点会做什么',
      '- 扫描并处理 `UpdateOrchestrator` 与 `WaaSMedic` 相关计划任务文件。',
      '- 将任务设为禁用，并修改任务命令前缀（`DISABLE:`）降低自动恢复能力。',
      '- 释放时按备份还原命令与启用状态。',
      '',
      '### 需要动用的权限级别',
      '- 需要 **TrustedInstaller + 管理员** 权限。',
      '- 原因：任务定义文件在系统目录下，涉及系统任务写回与权限保护。',
    ].join('\n'),
  },
  fallback_settings: {
    title: '兜底更新设置',
    markdown: [
      '### 这个阻断点会做什么',
      '- 施加一组“兜底”更新参数：长暂停、禁驱动更新、抑制自动重启等。',
      '- 即便上层策略被改动，这组设置仍可在行为层继续压制更新动作。',
      '- 作为最后一道“配置兜底”防线。',
      '',
      '### 需要动用的权限级别',
      '- 需要 **TrustedInstaller + 管理员** 权限。',
      '- 原因：涉及多处受保护系统更新注册表项，需要稳定高权限写入。',
    ].join('\n'),
  },
  extreme_mode: {
    title: '极端手段',
    markdown: [
      '### 这个阻断点会做什么',
      '- 强删 SoftwareDistribution 目录，然后用同名文件站桩、变更所有者、删除一切继承来的权限。',
      '- **会导致微软商店也停止工作！**',
      '',
      '### 需要动用的权限级别',
      '- 需要 **TrustedInstaller + 管理员** 权限。',
      '- 原因：涉及系统目录删除/替换和所有权处理，属于高风险高权限操作。',
    ].join('\n'),
  },
}

export function getGuardPrinciple(pointId: string): GuardPrinciple {
  return (
    PRINCIPLES[pointId] || {
      title: '技术原理',
      markdown: '暂无说明。',
    }
  )
}
