//! # afastdata 基础示例 / Basic Example
//!
//! 演示 afastdata 序列化框架的核心功能，包括：
//! - 结构体序列化/反序列化（含 String、u32、i32、Vec、Option 字段）
//! - 枚举序列化/反序列化（含 unit、tuple、named-field 变体）
//! - 嵌套复杂结构体的 round-trip 测试
//!
//! Demonstrates the core features of the afastdata serialization framework:
//! - Struct serialization/deserialization (with String, u32, i32, Vec, Option fields)
//! - Enum serialization/deserialization (with unit, tuple, named-field variants)
//! - Round-trip testing of nested complex structures
//!
//! ## 运行 / Run
//!
//! ```bash
//! cargo run --example basic -p afastdata
//! ```

use afastdata::{AFastDeserialize, AFastSerialize};

/// 玩家数据结构。包含名称、等级、生命值、技能列表和公会信息。
///
/// Player data structure. Contains name, level, HP, skill list, and guild info.
///
/// # 编码格式 / Encoding Format
///
/// 字段按声明顺序依次序列化：
/// Fields are serialized in declaration order:
/// 1. `name` — `LenInt` 长度前缀 + UTF-8 字节 / length prefix + UTF-8 bytes
/// 2. `level` — 4 字节 little-endian / 4 bytes little-endian
/// 3. `hp` — 4 字节 little-endian / 4 bytes little-endian
/// 4. `skills` — `LenInt` 元素个数 + 逐个 String / element count + each String
/// 5. `guild` — 1 字节标记 + (若 Some) String 数据 / 1 byte tag + (if Some) String data
#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Player {
    /// 玩家名称 / Player name
    name: String,
    /// 等级 / Level
    level: u32,
    /// 生命值 / Hit points
    hp: i32,
    /// 技能列表 / Skill list
    skills: Vec<String>,
    /// 公会名称（可选）/ Guild name (optional)
    guild: Option<String>,
    /// Other
    #[afast(skip("default"))]
    other: i32,
}

fn default() -> i32 {
    123
}

/// 游戏物品枚举。展示枚举的三种典型变体形式。
///
/// Game item enum. Demonstrates three typical variant forms of enums.
///
/// # 编码格式 / Encoding Format
///
/// 每个变体以 `u32` 索引开头（从 0 开始）：
/// Each variant starts with a `u32` index (starting from 0):
/// - `Weapon` — 索引 0 + `name`(String) + `damage`(i32)
/// - `Potion` — 索引 1 + `heal`(i32)
/// - `Gold` — 索引 2 + 金额(u64)
#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Item {
    /// 武器：名称 + 伤害值
    /// Weapon: name + damage value
    Weapon { name: String, damage: i32 },
    /// 药水：回复量
    /// Potion: heal amount
    Potion { heal: i32 },
    /// 金币：数量
    /// Gold: amount
    Gold(u64),
}

/// 背包结构体。包含所有者、物品列表和容量上限。
///
/// Inventory structure. Contains owner, item list, and capacity limit.
///
/// 用于演示嵌套复杂结构体的序列化能力。
///
/// Used to demonstrate the serialization capability for nested complex structures.
#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Inventory {
    /// 背包所有者 / Inventory owner
    owner: String,
    /// 物品列表 / Item list
    items: Vec<Item>,
    /// 容量上限 / Capacity limit
    capacity: u32,
}

/// 泛型 round-trip 函数：接受任意同时实现了 AFastSerialize 和 AFastDeserialize 的类型。
///
/// Generic round-trip function: accepts any type that implements both AFastSerialize
/// and AFastDeserialize.
fn roundtrip<T: AFastSerialize + AFastDeserialize + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = val.to_bytes();
    let (decoded, size) = T::from_bytes(&bytes).unwrap();
    assert_eq!(size, bytes.len());
    assert_eq!(*val, decoded);
    println!("{:?} round-trip OK ({} bytes)", val, size);
}

fn main() {
    // ==================== 示例 1：基础结构体序列化 / Example 1: Basic Struct Serialization ====================

    let player = Player {
        name: String::from("Alice"),
        level: 42,
        hp: 850,
        skills: vec![String::from("fireball"), String::from("shield")],
        guild: Some(String::from("Dragon Slayers")),
        other: 123,
    };

    // 序列化为字节数组 / Serialize to byte array
    let bytes = player.to_bytes();
    println!("Player 序列化后 {} 字节: {:?}", bytes.len(), bytes);

    // 从字节数组反序列化 / Deserialize from byte array
    let (decoded, size) = Player::from_bytes(&bytes).unwrap();
    println!("Player 反序列化后: {:?}", decoded);
    println!("消耗字节数: {}", size);

    // 验证 round-trip 一致性 / Verify round-trip consistency
    assert_eq!(player, decoded);
    println!("Player round-trip OK\n");

    // ==================== 示例 2：枚举序列化 / Example 2: Enum Serialization ====================

    let items = vec![
        Item::Weapon {
            name: String::from("Excalibur"),
            damage: 999,
        },
        Item::Potion { heal: 50 },
        Item::Gold(10000),
    ];

    // 序列化枚举向量 / Serialize enum vector
    let bytes = items.to_bytes();
    println!("Items 序列化后 {} 字节: {:?}", bytes.len(), bytes);

    // 反序列化枚举向量 / Deserialize enum vector
    let (decoded, _) = Vec::<Item>::from_bytes(&bytes).unwrap();
    println!("Items 反序列化后: {:?}", decoded);
    assert_eq!(items, decoded);
    println!("Items round-trip OK\n");

    // ==================== 示例 3：嵌套复杂结构 / Example 3: Nested Complex Structure ====================

    let inventory = Inventory {
        owner: String::from("Bob"),
        items: vec![
            Item::Weapon {
                name: String::from("Dagger"),
                damage: 25,
            },
            Item::Gold(500),
        ],
        capacity: 20,
    };

    // 序列化嵌套结构 / Serialize nested structure
    let bytes = inventory.to_bytes();
    println!("Inventory 序列化后 {} 字节: {:?}", bytes.len(), bytes);

    // 反序列化嵌套结构 / Deserialize nested structure
    let (decoded, _) = Inventory::from_bytes(&bytes).unwrap();
    println!("Inventory 反序列化后: {:?}", decoded);
    assert_eq!(inventory, decoded);
    println!("Inventory round-trip OK\n");

    // ==================== 示例 4：Option 为 None 的情况 / Example 4: Option with None ====================

    let lonely = Player {
        name: String::from("Charlie"),
        level: 1,
        hp: 100,
        skills: vec![],
        guild: None,
        other: 123,
    };

    let bytes = lonely.to_bytes();
    let (decoded, _) = Player::from_bytes(&bytes).unwrap();
    assert_eq!(lonely, decoded);
    println!("Option::None round-trip OK");

    // ==================== 示例 5：泛型函数 / Example 5: Generic Function ====================

    roundtrip(&player);
    roundtrip(&inventory);
    roundtrip(&Item::Gold(42));
    roundtrip(&vec![1i32, 2, 3]);
    roundtrip(&Some(String::from("hello")));
    roundtrip(&42u32);

    println!("\nAll examples passed!");
}
