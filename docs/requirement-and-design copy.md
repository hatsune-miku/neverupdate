# NeverUpdate (NU)

目前，用户在拥有自己的电子设备所有权的情况下，却无法决定自己的设备是否更新系统，这显然是不合理的——用户使用购买电脑时所附送的 Windows 系统，不代表用户默许 Microsoft 在不经过用户同意的情况下，向用户所有的设备中写入数据、执行重启、安装软件。

NeverUpdate 将做一些非常广泛的尝试，达成一个固定目标：**防止用户的 Windows 11 系统出现自动更新的行为，同时确保微软商店可以正常使用。**

具体来说，NeverUpdate 将是一个面向零售版本 Windows 11 的自动更新阻断工具，它将用户被剥夺的“不更新系统”的权利归还给用户。

# 顶层设计

NU 分为核心 core 模块、daemon 模块和 GUI 模块。

### daemon 模块

daemon 模块是一个具有 Administrator 权限的后台守护进程，技术上必要的话，可以直接注册为系统服务。

daemon 模块的主要职责是：

#### 前置系统检查

在 NU 运行之前，必须检查系统是否满足以下条件：

- 确保系统是 Windows 11
- 确保系统不是 Windows Server
- 确保系统不是 LTSC 版本，这样的版本无需 NU 干预
- 确保自身拥有管理员权限
- 确保管理员权限测试通过，自己真的有管理员权限
- 确保自身是单一实例

#### 阻断更新服务的守护服务，减少其对 NU 的干扰

定期周期性检查以下服务。

- Windows Update Medic Service
- Update Orchestrator Service
- Windows Update Health Service

以上服务将通过注册表进行禁用处理，并修改 ImagePath，在原始路径的基础上，添加 DISABLE 前缀，使其成为无效路径。

#### 调整 Hosts 和防火墙设置

定期周期性检查 Hosts 和系统防火墙规则，确保：

- \*.windowsupdate.com
- \*.update.microsoft.com
- \*.delivery.mp.microsoft.com

这些域名存在于 Hosts 文件并解析为 127.0.0.1，并存在于系统防火墙的拒绝规则中，确保这些域名不会被访问。

#### 调整组策略设置

定期周期性检查组策略设置，确保：

- "配置自动更新" - 禁用
- "删除使用所有 Windows 更新功能的访问权限" - 启用

你需要查阅资料来确定具体的路径，不要猜测。

#### 管理计划任务

- \Microsoft\Windows\UpdateOrchestrator\
- \Microsoft\Windows\WaaSMedic\

以上计划任务将通过注册表进行禁用处理，并修改 ImagePath，在原始路径的基础上，添加 DISABLE 前缀，使其成为无效路径。

#### 调整 Windows 更新设置，做最后的兜底处理

定期周期性检查以下设置。

- 推迟 Windows 更新到极晚的时间点。参考：https://github.com/Aetherinox/pause-windows-updates/blob/main/windows-updates-pause.reg
- 禁止系统在更新失败时自动重试
- 禁止更新驱动、固件、可选组件

### GUI 模块

#### 用户界面

GUI 模块将提供一个用户界面，用户可以在此界面中：

- 程序是否拥有管理员权限，如果无管理员权限，则提示用户以管理员身份运行。
- 查看各个阻断更新的检查点是否"失守"
- 每个阻断点，有阻断、放开、修复三种操作。其中修复仅限"失守"的检查点可以执行。
- 提供一键阻断、一键放开、一键修复所有阻断点的功能。
- 查看守护进程是否运行正常。
- 注册、重新注册守护程序为系统服务。
- 对于确实无需微软商店的用户，经过二次确认后，提供执行极端手段（见下文）的入口。
- 用户可以查看各个阻断点的历史记录，包括：
  - 阻断时间
  - 放开时间
  - 修复时间

此外，GUI 模块具有以下特性：

- 义务性提醒。初次运行时，告知此工具明确的危险性，滑动到底部后，才可以点击继续。

#### 极端手段

如果用户明确自己的需求，认为自己不需要微软商店，则可以提供入口，实现更加极端的处理：

- Windows Update Medic Service
- Update Orchestrator Service
- Windows Update Health Service
- Windows Update

以上服务将通过注册表进行禁用处理，并修改 ImagePath，在原始路径的基础上，添加 DISABLE 前缀，使其成为无效路径。

然后，定位 SoftwareDistribution 目录：`%SystemRoot%\SoftwareDistribution`，并删除该目录，并在原位置创建一个非空的同名文件、变更所有者为用户当前用户、删除一切继承来的权限。

### core 模块

显然，daemon 和 GUI 有着相当大的功能重叠，core 模块将作为这两个模块的共享模块：

- 尽量抽象化，减少重复代码。

这就是 core 模块唯一的需求了，你需要分析这两个模块的共同点，抽象出 core 模块。

## 从现有类似架构项目中，继承通用功能

你可以参考我的另一个项目：https://github.com/hatsune-miku/kook-kvm

这个项目和本项目在核心功能上毫无关系，但是架构却非常相似。需要你从 kvm 项目中，继承：

- UI 设计风格
- Tauri 架构
- 自动更新能力。域名你可以先照搬 kvm 项目的，后续我替换
- KVM 以附件方式附带 CLI 工具的特性。对本项目来说，可以用相同方式附带守护进程
- 大体文件结构

# 技术细节

你需要一开始就做到：

- 坚持模块化设计。不要让单个文件变得过于庞大。
- 坚持单一职责原则。不要让单个文件承担过多的职责。
- 坚持 DRY 原则。对于重复第 2 遍的代码，不用犹豫，抽象出来。
- 敢于重构甚至推翻。在编码过程中发现设计不合理，或者代码写得不好看，不要犹豫，重构甚至推翻。
- 注意性能。减少轮询操作。
- 使用 Tauri 和 Bun 作为开发框架和运行时/包管理。
- 使用 TypeScript (React 19 + Scss) 和 Rust。

## 编码规范

### Rust

- 绝对禁止使用 `panic!`。你应当总是在返回值中表达错误。
- 优先保证包体小，慎重引入 tokio 这种强大但是复杂的库，尽量不要全量引入。
- 在必要的时候，善用 unsafe，不需要围绕语言特性做过于复杂的逻辑。
- 尽量避免出现生命周期传染。
- 对于涉及 Windows API 调用的变量，或是对于冠以 `DWORD` 等具有 Windows 特色的类型的变量，使用匈牙利命名法。对于平常 Rust 逻辑的变量，遵循 RFC 430 (https://rust-lang.github.io/api-guidelines/naming.html)。

  在使用匈牙利命名法的过程中，存在以下偏好：

  - `sz` 描述 zero-terminated string
  - `s` 描述 Rust string
  - `n` 描述 number，而不是 `i`。举例：`nRet` 而不是 `iRet`
  - `b` 描述 boolean，而不是 `f`。举例：`bRet` 而不是 `fRet`
  - `p` 描述 pointer 的同时，也应当用于语言意义上的数组（总之是连续内存的起点），和函数。

- 善用宏定义。

### TypeScript & CSS

- 绝对禁止使用 `throw`。
- 避免使用 `try` 和 `catch`，只是有的时候三方库会抛异常，你才可以被迫使用 `try` 和 `catch`。
- 你无需避免 `null` 和 `undefined`，而是应该以正确语义来善用它们。
- 不要通过 `!!` 写法来转换布尔值。
- 不要通过 `cond && foo()` 写法来实现分支逻辑。
- 善用 `?.`, `||`, `??`, `?.()` 等语法糖。
- 使用 `function` 定义 function。
- 对于 boolean 类型，避免 `isFoo` 命名，而是省略 `is`，直接写 `foo`。
- 使用 Prettier 进行代码格式化。

  - 使用 2 空格作为缩进
  - 使用单引号
  - 使用 200 字符作为换行
  - 省略分号
  - 引入 import 自动排序。具体来说：

    ```
    importOrder: [
        // 1. react 相关
        '^react(-.*)?$',

        // 2. 其他第三方库 字母开头
        '^[a-zA-Z]',

        // 3. @/ 开头的 alias路径包
        '^@',

        // 4. @任意字母
        '^@[a-zA-Z]',

        // 5. 相对路径 ./（排除 .scss）
        '^\\.\\/(?!.*\\.(css|scss)$).*',

        // 6. 相对路径 ../（排除 .scss）
        '^\\.\\.\\/(?!.*\\.(css|scss)$).*',

        // 7. .scss 文件
        '\\.(css|scss)$',
    ]
    ```

- 禁止出现注释和代码在同一行的写法。具体来说，你应该避免：

  ```ts
  const foo = 'foo' // Some comment
  const bar = 'bar' // Some comment
  ```

  而是应该写成：

  ```ts
  // Some comment
  const foo = 'foo'

  // Some comment
  const bar = 'bar'
  ```

- 避免 `enum` 关键字，使用 `as const` 和 `type` 来代替。具体来说，你应该避免：

  ```ts
  // 避免此种写法
  export enum ModelKind {
    anthropic = 'anthropic',
    openai = 'openai',
  }
  ```

  作为替代，你应该写：

  ```ts
  // 提倡此种写法
  export const ModelKinds = ['anthropic', 'openai'] as const
  export type ModelKind = (typeof ModelKinds)[keyof typeof ModelKinds]
  ```

- 避免创建无名类型。具体来说，你应该避免：

  ```ts
  function foo({ data }: { data: string }): { code: number } {
    return {
      code: 0,
    }
  }
  ```

  而是应该写：

  ```ts
  interface Data {
    data: string
  }

  interface Response {
    code: number
  }

  function foo({ data }: Data): Response {
    return {
      code: 0,
    }
  }
  ```

- 你可以善用 `any`。使用 `any` 时，你可以通过直接省略类型定义来暗示这是 `any` 类型。
- 妥善配置，使用 `@/` 代表项目根目录，指向上级时使用。
- 使用 `import { FooComponent } from './FooComponent'`，而不是 `import { FooComponent } from './FooComponent.js'`。不要使用那些必须写 `.js` 后缀的模块系统。
- 对于组件，使用：

  ```

  FooComponent/ - index.tsx - index.scss

  ```

  而不是：

  ```

  FooComponent.tsx
  FooComponent.scss

  ```

- 使用 zustand 作为全局状态管理，如果你认为需要一个全局状态管理的话。
- 避免自行创造 React Context 实现状态管理。
- 避免使用 Tailwind CSS。
- 充分利用 Scss 的优势，写：

  ```scss
  .container {
    color: blue;
    .enclosure {
      color: red;
    }
  }
  ```

  instead of:

  ```scss
  .container {
    color: blue;
  }

  .container .enclosure {
    color: red;
  }
  ```

- 避免 inline styles，但如果仅设置 `backgroundImage`，则可以。
- 避免 BEM 命名方式。写 `.foo-bar-title` 而不是 `.foo__bar__title`。
- 可以善用 `!important` 来覆盖样式，但不要滥用。
- 动画曲线使用 `cubic-bezier(0.29, 0, 0, 1)`。
- 在项目中，已有 `eslint-partial.config.mjs`，你应当遵守里面的规则定义。
- 在项目中，已有 `tsconfig-partial.json`，你应当遵守里面的规则定义。

# 文档需求

- 本次需求中涉及的各种规范，需总结到一篇后续给 AI 读的文档中，放在 `public-docs/conventions.md` 中。文档中只包含通用规范，不包含任何与 NU 这一具体工程相关的内容。

- 需要你创建一个合适的 `.gitignore` 文件。
