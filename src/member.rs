use chrono::NaiveDate;

pub type StudentId = u32;

pub enum MemberType {
    Student,
}

// impl<S> TryFrom<S> for MemberType
// where
//     S: AsRef<str>,
// {
//     fn try_from(value: S) -> Result<Self, Self::Error> {
//         match value {
//             "Student" => Ok(Self::Student),
//             _ => Err(()),
//         }
//     }
// }

pub struct Member {
    student_id: StudentId,
    name: String,
    member_type: MemberType,
    subscription_purchased: String,
    date_joined: NaiveDate,
}
