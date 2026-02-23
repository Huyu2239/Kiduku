use crate::usecase::dto::{HelpCommandDto, HelpOutputDto, UsecaseError};

pub fn execute() -> Result<HelpOutputDto, UsecaseError> {
    let commands = vec![
        HelpCommandDto {
            name: "/help".into(),
            description: "Botの使い方を表示します。".into(),
            example: "/help".into(),
        },
        HelpCommandDto {
            name: "既読状況確認 (メッセージ右クリック)".into(),
            description: "メッセージの既読・未読・解決済みユーザーを確認します。".into(),
            example: "メッセージを右クリック → アプリ → 既読状況確認".into(),
        },
        HelpCommandDto {
            name: "/通知一覧".into(),
            description: "自分宛のメンション一覧を確認します。show_done で解決済みも表示できます。"
                .into(),
            example: "/通知一覧 show_done:true".into(),
        },
    ];

    Ok(HelpOutputDto {
        title: "Kiduku ヘルプ".into(),
        description: "メンション付きメッセージに✅リアクションを付与し、既読状況を可視化します。"
            .into(),
        commands,
    })
}
