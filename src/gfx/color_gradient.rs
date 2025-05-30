
pub struct ColorGradient
{
    start: [f32; 4],
    gradient: [f32; 4],
}

impl ColorGradient
{
    pub fn new(start: [f32; 4], end [f32; 4]) -> Self
    {
        let gradient = [
            end[0] - start[0],
            end[1] - start[1],
            end[2] - start[2],
            end[3] - start[3],
        ];

        ColorGradient {
            start,
            gradient,
        }
    }


    pub fn new_with_gradient(start: [f32; 4], gradient [f32; 4]) -> Self
    {
        ColorGradient {
            start,
            gradient,
        }
    }

    pub fn color_at(&self, p: f32) -> [f32; 4]
    {
        [
            self.start[0] + self.gradient[0] * p,
            self.start[1] + self.gradient[1] * p,
            self.start[2] + self.gradient[2] * p,
            self.start[3] + self.gradient[3] * p,
        ]
    }

}